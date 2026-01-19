//! Source URL parsing and normalization.

use crate::SkiloError;
use std::path::PathBuf;
use url::Url;

/// A parsed source for skills - either a git repository or a local path.
#[derive(Debug, Clone)]
pub enum Source {
    /// A git repository URL.
    Git(GitSource),
    /// A local filesystem path.
    Local(PathBuf),
}

/// A parsed git repository source.
#[derive(Debug, Clone)]
pub struct GitSource {
    /// The normalized git URL (HTTPS or SSH).
    pub url: String,
    /// The optional branch to checkout.
    pub branch: Option<String>,
    /// The optional tag to checkout.
    pub tag: Option<String>,
    /// The optional subdirectory within the repository.
    pub subdir: Option<String>,
}

impl Source {
    /// Parse a source string into a Source enum.
    ///
    /// Supports:
    /// - GitHub shorthand: `owner/repo`
    /// - Full GitHub URL: `https://github.com/owner/repo`
    /// - GitLab URL: `https://gitlab.com/owner/repo`
    /// - SSH URL: `git@github.com:owner/repo.git`
    /// - Direct skill path: `https://github.com/owner/repo/tree/main/skills/my-skill`
    /// - Local path: `./path/to/skills` or `/absolute/path`
    pub fn parse(source: &str) -> Result<Self, SkiloError> {
        // Check for local path first
        if source.starts_with('/')
            || source.starts_with("./")
            || source.starts_with("../")
            || source.starts_with('~')
        {
            return Ok(Source::Local(PathBuf::from(source)));
        }

        // Check for SSH URL: git@host:owner/repo.git
        if source.starts_with("git@") {
            return Self::parse_ssh_url(source);
        }

        // Check for full URL
        if source.starts_with("http://") || source.starts_with("https://") {
            return Self::parse_https_url(source);
        }

        // Check for GitHub shorthand: owner/repo
        if Self::is_github_shorthand(source) {
            return Ok(Source::Git(GitSource {
                url: format!("https://github.com/{}.git", source),
                branch: None,
                tag: None,
                subdir: None,
            }));
        }

        Err(SkiloError::InvalidSource(
            source.to_string(),
            "Expected: owner/repo, https://github.com/owner/repo, git@github.com:owner/repo.git, or local path".to_string(),
        ))
    }

    /// Parse a source string with optional branch/tag overrides.
    pub fn parse_with_options(
        source: &str,
        branch: Option<String>,
        tag: Option<String>,
    ) -> Result<Self, SkiloError> {
        let mut result = Self::parse(source)?;

        if let Source::Git(ref mut git) = result {
            if branch.is_some() {
                git.branch = branch;
            }
            if tag.is_some() {
                git.tag = tag;
            }
        }

        Ok(result)
    }

    fn is_github_shorthand(s: &str) -> bool {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        let is_valid_name = |name: &str| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        };

        is_valid_name(parts[0]) && is_valid_name(parts[1])
    }

    fn parse_ssh_url(source: &str) -> Result<Self, SkiloError> {
        // git@github.com:owner/repo.git
        let rest = source.strip_prefix("git@").ok_or_else(|| {
            SkiloError::InvalidSource(source.to_string(), "Invalid SSH URL format".to_string())
        })?;

        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(SkiloError::InvalidSource(
                source.to_string(),
                "SSH URL must be in format git@host:owner/repo.git".to_string(),
            ));
        }

        let host = parts[0];
        let path = parts[1].trim_end_matches(".git");

        Ok(Source::Git(GitSource {
            url: format!("git@{}:{}.git", host, path),
            branch: None,
            tag: None,
            subdir: None,
        }))
    }

    fn parse_https_url(source: &str) -> Result<Self, SkiloError> {
        let url = Url::parse(source).map_err(|_| {
            SkiloError::InvalidSource(source.to_string(), "Invalid URL format".to_string())
        })?;

        let host = url.host_str().ok_or_else(|| {
            SkiloError::InvalidSource(source.to_string(), "URL must have a host".to_string())
        })?;

        let path = url.path().trim_start_matches('/').trim_end_matches(".git");

        // Check for tree/branch/path format: owner/repo/tree/branch/path
        if let Some(tree_idx) = path.find("/tree/") {
            let repo_path = &path[..tree_idx];
            let rest = &path[tree_idx + 6..]; // skip "/tree/"

            // Split into branch and optional subdir
            let (branch, subdir) = if let Some(slash_idx) = rest.find('/') {
                let branch = &rest[..slash_idx];
                let subdir = &rest[slash_idx + 1..];
                (
                    Some(branch.to_string()),
                    if subdir.is_empty() {
                        None
                    } else {
                        Some(subdir.to_string())
                    },
                )
            } else {
                (Some(rest.to_string()), None)
            };

            return Ok(Source::Git(GitSource {
                url: format!("https://{}/{}.git", host, repo_path),
                branch,
                tag: None,
                subdir,
            }));
        }

        // Standard repo URL
        Ok(Source::Git(GitSource {
            url: format!("https://{}/{}.git", host, path),
            branch: None,
            tag: None,
            subdir: None,
        }))
    }
}

impl GitSource {
    /// Get the reference to checkout (branch, tag, or HEAD).
    pub fn reference(&self) -> Option<&str> {
        self.branch.as_deref().or(self.tag.as_deref())
    }

    /// Get a display-friendly name for the source.
    pub fn display_name(&self) -> String {
        // Extract owner/repo from URL
        let url = self.url.trim_end_matches(".git");
        if let Some(idx) = url.rfind("://") {
            let path = &url[idx + 3..];
            if let Some(slash_idx) = path.find('/') {
                return path[slash_idx + 1..].to_string();
            }
        }
        if url.starts_with("git@") {
            if let Some(idx) = url.find(':') {
                return url[idx + 1..].to_string();
            }
        }
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_shorthand() {
        let source = Source::parse("owner/repo").unwrap();
        if let Source::Git(git) = source {
            assert_eq!(git.url, "https://github.com/owner/repo.git");
            assert!(git.branch.is_none());
            assert!(git.subdir.is_none());
        } else {
            panic!("Expected Git source");
        }
    }

    #[test]
    fn test_parse_github_url() {
        let source = Source::parse("https://github.com/owner/repo").unwrap();
        if let Source::Git(git) = source {
            assert_eq!(git.url, "https://github.com/owner/repo.git");
        } else {
            panic!("Expected Git source");
        }
    }

    #[test]
    fn test_parse_github_url_with_tree() {
        let source =
            Source::parse("https://github.com/owner/repo/tree/main/skills/my-skill").unwrap();
        if let Source::Git(git) = source {
            assert_eq!(git.url, "https://github.com/owner/repo.git");
            assert_eq!(git.branch, Some("main".to_string()));
            assert_eq!(git.subdir, Some("skills/my-skill".to_string()));
        } else {
            panic!("Expected Git source");
        }
    }

    #[test]
    fn test_parse_ssh_url() {
        let source = Source::parse("git@github.com:owner/repo.git").unwrap();
        if let Source::Git(git) = source {
            assert_eq!(git.url, "git@github.com:owner/repo.git");
        } else {
            panic!("Expected Git source");
        }
    }

    #[test]
    fn test_parse_local_path() {
        let source = Source::parse("./path/to/skills").unwrap();
        if let Source::Local(path) = source {
            assert_eq!(path, PathBuf::from("./path/to/skills"));
        } else {
            panic!("Expected Local source");
        }
    }

    #[test]
    fn test_parse_absolute_path() {
        let source = Source::parse("/absolute/path").unwrap();
        if let Source::Local(path) = source {
            assert_eq!(path, PathBuf::from("/absolute/path"));
        } else {
            panic!("Expected Local source");
        }
    }

    #[test]
    fn test_display_name() {
        let git = GitSource {
            url: "https://github.com/owner/repo.git".to_string(),
            branch: None,
            tag: None,
            subdir: None,
        };
        assert_eq!(git.display_name(), "owner/repo");
    }
}
