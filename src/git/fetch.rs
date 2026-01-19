//! Git repository fetching operations.

use crate::git::source::GitSource;
use crate::SkiloError;
use git2::{build::RepoBuilder, FetchOptions, Repository};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Result of a successful fetch operation.
pub struct FetchResult {
    /// The temporary directory containing the cloned repository.
    pub temp_dir: TempDir,
    /// The path to the root of the repository (or subdir if specified).
    pub root: PathBuf,
}

/// Fetch a git repository to a temporary directory.
pub fn fetch(source: &GitSource) -> Result<FetchResult, SkiloError> {
    let temp_dir = TempDir::new().map_err(SkiloError::Io)?;

    clone_repo(&source.url, source.reference(), temp_dir.path())?;

    // Determine the root path (may be a subdirectory)
    let root = if let Some(ref subdir) = source.subdir {
        temp_dir.path().join(subdir)
    } else {
        temp_dir.path().to_path_buf()
    };

    if !root.exists() {
        return Err(SkiloError::InvalidSource(
            source.url.clone(),
            format!(
                "Subdirectory '{}' not found in repository",
                source.subdir.as_deref().unwrap_or("")
            ),
        ));
    }

    Ok(FetchResult { temp_dir, root })
}

fn clone_repo(url: &str, reference: Option<&str>, dest: &Path) -> Result<Repository, SkiloError> {
    let mut builder = RepoBuilder::new();

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.depth(1);
    builder.fetch_options(fetch_opts);

    if let Some(ref_name) = reference {
        builder.branch(ref_name);
    }

    builder.clone(url, dest).map_err(|e| {
        let message = e.message().to_string();

        if message.contains("not found")
            || message.contains("404")
            || message.contains("Repository not found")
        {
            SkiloError::RepoNotFound {
                url: url.to_string(),
            }
        } else if message.contains("Could not resolve host")
            || message.contains("network")
            || message.contains("connection")
        {
            SkiloError::Network { message }
        } else {
            SkiloError::Git { message }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_nonexistent_repo() {
        let source = GitSource {
            url: "https://github.com/nonexistent-owner-xyz/nonexistent-repo-xyz.git".to_string(),
            branch: None,
            tag: None,
            subdir: None,
        };

        let result = fetch(&source);
        assert!(result.is_err());
    }
}
