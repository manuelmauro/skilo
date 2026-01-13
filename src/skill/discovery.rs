//! Skill discovery utilities.

use crate::skill::manifest::{Manifest, ManifestError};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Utility for discovering skills in the filesystem.
pub struct Discovery;

impl Discovery {
    /// Find all SKILL.md files in a directory tree.
    pub fn find_skills(root: &Path) -> Vec<PathBuf> {
        // If root is a SKILL.md file, return it directly
        if root.is_file() && root.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
            return vec![root.to_path_buf()];
        }

        // If root contains a SKILL.md, return just that
        let skill_md = root.join("SKILL.md");
        if skill_md.exists() {
            return vec![skill_md];
        }

        // Otherwise, search recursively
        WalkDir::new(root)
            .follow_links(true)
            .into_iter()
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
    pub fn discover(root: &Path) -> Vec<Result<Manifest, (PathBuf, ManifestError)>> {
        let paths = Self::find_skills(root);
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

        let skills = Discovery::find_skills(&skill_dir);
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

        let skills = Discovery::find_skills(temp.path());
        assert_eq!(skills.len(), 3);
    }
}
