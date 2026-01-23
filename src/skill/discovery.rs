//! Skill discovery utilities.

use crate::skill::manifest::{Manifest, ManifestError};
use globset::{Glob, GlobSetBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Utility for discovering skills in the filesystem.
pub struct Discovery;

impl Discovery {
    /// Find all SKILL.md files in a directory tree.
    ///
    /// The `ignore_patterns` parameter specifies glob patterns for directories to skip during traversal.
    /// Patterns follow `.gitignore` style glob syntax and can match directory names or paths:
    /// - `target` - matches any directory named "target"
    /// - `build-*` - matches directories starting with "build-"
    /// - `foo/bar` - matches the path "foo/bar" relative to search root
    /// - `**/cache` - matches "cache" at any depth
    pub fn find_skills(root: &Path, ignore_patterns: &[String]) -> Vec<PathBuf> {
        // If root is a SKILL.md file, return it directly
        if root.is_file() && root.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
            return vec![root.to_path_buf()];
        }

        // If root contains a SKILL.md, return just that
        let skill_md = root.join("SKILL.md");
        if skill_md.exists() {
            return vec![skill_md];
        }

        // Build a GlobSet from ignore patterns
        let mut builder = GlobSetBuilder::new();
        for pattern in ignore_patterns {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        let globset = builder
            .build()
            .unwrap_or_else(|_| GlobSetBuilder::new().build().unwrap());

        // Otherwise, search recursively, skipping ignored directories
        WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                // Allow the root directory itself
                if e.path() == root {
                    return true;
                }

                // Skip ignored directories using glob matching against relative path
                if e.file_type().is_dir() {
                    // Get relative path from root for matching
                    if let Ok(rel_path) = e.path().strip_prefix(root) {
                        // Match against both the relative path and just the directory name
                        // This supports both "target" and "foo/bar" style patterns
                        let path_str = rel_path.to_string_lossy();
                        if globset.is_match(path_str.as_ref()) {
                            return false;
                        }

                        // Also check just the directory name for simple patterns
                        if let Some(name) = e.file_name().to_str() {
                            if globset.is_match(name) {
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "SKILL.md")
            .map(|e| e.into_path())
            .collect()
    }

    /// Load all skills from a list of paths.
    pub fn load_skills(paths: &[PathBuf]) -> Vec<Result<Manifest, (PathBuf, ManifestError)>> {
        paths
            .iter()
            .map(|path| Manifest::parse(path.clone()).map_err(|e| (path.clone(), e)))
            .collect()
    }

    /// Find and load all skills in a directory.
    pub fn discover(
        root: &Path,
        ignore_patterns: &[String],
    ) -> Vec<Result<Manifest, (PathBuf, ManifestError)>> {
        let paths = Self::find_skills(root, ignore_patterns);
        Self::load_skills(&paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_single_skill() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("my-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: test\n---\n",
        )
        .unwrap();

        let skills = Discovery::find_skills(&skill_dir, &[]);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].ends_with("SKILL.md"));
    }

    #[test]
    fn test_find_multiple_skills() {
        let temp = TempDir::new().unwrap();

        for name in ["skill-a", "skill-b", "skill-c"] {
            let skill_dir = temp.path().join(name);
            fs::create_dir(&skill_dir).unwrap();
            fs::write(
                skill_dir.join("SKILL.md"),
                format!("---\nname: {}\ndescription: test\n---\n", name),
            )
            .unwrap();
        }

        let skills = Discovery::find_skills(temp.path(), &[]);
        assert_eq!(skills.len(), 3);
    }

    #[test]
    fn test_find_skills_with_ignore() {
        let temp = TempDir::new().unwrap();

        // Create skills in different directories
        for name in ["skill-a", "skill-b"] {
            let skill_dir = temp.path().join(name);
            fs::create_dir(&skill_dir).unwrap();
            fs::write(
                skill_dir.join("SKILL.md"),
                format!("---\nname: {}\ndescription: test\n---\n", name),
            )
            .unwrap();
        }

        // Create a skill in target directory
        let target_dir = temp.path().join("target");
        fs::create_dir(&target_dir).unwrap();
        let target_skill = target_dir.join("ignored-skill");
        fs::create_dir(&target_skill).unwrap();
        fs::write(
            target_skill.join("SKILL.md"),
            "---\nname: ignored\ndescription: test\n---\n",
        )
        .unwrap();

        let skills = Discovery::find_skills(temp.path(), &["target".to_string()]);
        assert_eq!(skills.len(), 2);
        assert!(skills
            .iter()
            .all(|p| !p.to_string_lossy().contains("target")));
    }

    #[test]
    fn test_find_skills_with_glob_wildcard() {
        let temp = TempDir::new().unwrap();

        // Create skills in different directories
        let skill_a = temp.path().join("skill-a");
        fs::create_dir(&skill_a).unwrap();
        fs::write(
            skill_a.join("SKILL.md"),
            "---\nname: skill-a\ndescription: test\n---\n",
        )
        .unwrap();

        // Create skills in build-* directories
        for name in ["build-debug", "build-release", "build-test"] {
            let build_dir = temp.path().join(name);
            fs::create_dir(&build_dir).unwrap();
            let skill_dir = build_dir.join("skill");
            fs::create_dir(&skill_dir).unwrap();
            fs::write(
                skill_dir.join("SKILL.md"),
                format!("---\nname: {}\ndescription: test\n---\n", name),
            )
            .unwrap();
        }

        // Use glob pattern to ignore all build-* directories
        let skills = Discovery::find_skills(temp.path(), &["build-*".to_string()]);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].to_string_lossy().contains("skill-a"));
        assert!(!skills
            .iter()
            .any(|p| p.to_string_lossy().contains("build-")));
    }

    #[test]
    fn test_find_skills_with_multiple_patterns() {
        let temp = TempDir::new().unwrap();

        // Create a valid skill
        let skill_dir = temp.path().join("my-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: test\n---\n",
        )
        .unwrap();

        // Create skills in various ignored directories
        for name in ["target", "node_modules", "dist", "build"] {
            let dir = temp.path().join(name);
            fs::create_dir(&dir).unwrap();
            let s = dir.join("skill");
            fs::create_dir(&s).unwrap();
            fs::write(
                s.join("SKILL.md"),
                format!("---\nname: {}\ndescription: test\n---\n", name),
            )
            .unwrap();
        }

        // Ignore multiple patterns
        let skills = Discovery::find_skills(
            temp.path(),
            &[
                "target".to_string(),
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
        );
        assert_eq!(skills.len(), 1);
        assert!(skills[0].to_string_lossy().contains("my-skill"));
    }

    #[test]
    fn test_find_skills_with_path_patterns() {
        let temp = TempDir::new().unwrap();

        // Create skill in root
        let skill_root = temp.path().join("skill-root");
        fs::create_dir(&skill_root).unwrap();
        fs::write(
            skill_root.join("SKILL.md"),
            "---\nname: skill-root\ndescription: test\n---\n",
        )
        .unwrap();

        // Create target/debug/skill
        let target_dir = temp.path().join("target");
        fs::create_dir(&target_dir).unwrap();
        let debug_dir = target_dir.join("debug");
        fs::create_dir(&debug_dir).unwrap();
        let debug_skill = debug_dir.join("skill");
        fs::create_dir(&debug_skill).unwrap();
        fs::write(
            debug_skill.join("SKILL.md"),
            "---\nname: debug-skill\ndescription: test\n---\n",
        )
        .unwrap();

        // Create target/release/skill (should also be ignored)
        let release_dir = target_dir.join("release");
        fs::create_dir(&release_dir).unwrap();
        let release_skill = release_dir.join("skill");
        fs::create_dir(&release_skill).unwrap();
        fs::write(
            release_skill.join("SKILL.md"),
            "---\nname: release-skill\ndescription: test\n---\n",
        )
        .unwrap();

        // Ignore target/debug specifically (should still find target/release)
        let skills = Discovery::find_skills(temp.path(), &["target/debug".to_string()]);
        assert_eq!(skills.len(), 2);
        assert!(skills
            .iter()
            .any(|p| p.to_string_lossy().contains("skill-root")));
        assert!(skills
            .iter()
            .any(|p| p.to_string_lossy().contains("release")));
        assert!(!skills.iter().any(|p| p.to_string_lossy().contains("debug")));

        // Ignore entire target directory with any subdirectory
        let skills = Discovery::find_skills(temp.path(), &["target/*".to_string()]);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].to_string_lossy().contains("skill-root"));
    }

    #[test]
    fn test_find_skills_with_deep_path_patterns() {
        let temp = TempDir::new().unwrap();

        // Create nested structure: foo/bar/baz/skill
        let foo = temp.path().join("foo");
        fs::create_dir(&foo).unwrap();
        let bar = foo.join("bar");
        fs::create_dir(&bar).unwrap();
        let baz = bar.join("baz");
        fs::create_dir(&baz).unwrap();
        let skill = baz.join("skill");
        fs::create_dir(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: nested\ndescription: test\n---\n",
        )
        .unwrap();

        // Create another skill at root level
        let root_skill = temp.path().join("root-skill");
        fs::create_dir(&root_skill).unwrap();
        fs::write(
            root_skill.join("SKILL.md"),
            "---\nname: root\ndescription: test\n---\n",
        )
        .unwrap();

        // Ignore foo/bar/baz specifically
        let skills = Discovery::find_skills(temp.path(), &["foo/bar/baz".to_string()]);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].to_string_lossy().contains("root-skill"));

        // Use ** pattern to match baz at any depth
        let skills = Discovery::find_skills(temp.path(), &["**/baz".to_string()]);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].to_string_lossy().contains("root-skill"));
    }
}
