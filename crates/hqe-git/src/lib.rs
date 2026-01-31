//! Git operations wrapper for HQE Workbench
//!
//! Provides safe git operations with dry-run support and error handling.

#![warn(missing_docs)]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use tracing::{debug, error, info, instrument};

/// Errors that can occur during git operations
#[derive(Debug, Error)]
pub enum GitError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Path canonicalization failed
    #[error("Failed to canonicalize path {path}: {source}")]
    PathCanonicalization {
        /// The path that failed to canonicalize
        path: PathBuf,
        /// The underlying error
        source: std::io::Error,
    },

    /// Not a git repository
    #[error("Not a git repository: {0}")]
    NotARepository(PathBuf),

    /// Git command failed
    #[error("Git command failed: {stderr}")]
    CommandFailed {
        /// Standard output
        stdout: String,
        /// Standard error
        stderr: String,
    },

    /// Clone operation failed
    #[error("Clone failed: {0}")]
    CloneFailed(String),

    /// Operation failed
    #[error("Failed to {operation}: {details}")]
    OperationFailed {
        /// The operation that failed
        operation: String,
        /// Details of the failure
        details: String,
    },
}

/// Git repository handle
#[derive(Debug, Clone)]
pub struct GitRepo {
    path: PathBuf,
}

/// Git operation result
/// Git operation result
#[derive(Debug, Clone)]
pub struct GitResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}

/// Branch information
/// Branch information
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Whether this is the current branch
    pub is_current: bool,
    /// Upstream branch name (if any)
    pub upstream: Option<String>,
}

/// Commit information
/// Commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Full commit hash
    pub hash: String,
    /// Short commit hash
    pub short_hash: String,
    /// Commit message
    pub message: String,
    /// Author name/email
    pub author: String,
    /// Commit date
    pub date: String,
}

impl GitRepo {
    /// Open a git repository at the given path
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, GitError> {
        let path = path.as_ref().to_path_buf();
        let canonical_path =
            tokio::fs::canonicalize(&path)
                .await
                .map_err(|e| GitError::PathCanonicalization {
                    path: path.clone(),
                    source: e,
                })?;

        // Verify it's a git repo
        let git_dir = canonical_path.join(".git");
        if tokio::fs::metadata(&git_dir).await.is_err() {
            return Err(GitError::NotARepository(canonical_path));
        }

        Ok(Self {
            path: canonical_path,
        })
    }

    /// Check if a path is inside a git repository
    pub async fn is_repo(path: impl AsRef<Path>) -> bool {
        let git_dir = path.as_ref().join(".git");
        tokio::fs::metadata(&git_dir).await.is_ok()
    }

    /// Get repository root path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Run a git command
    #[instrument(skip(self))]
    async fn run_git(&self, args: &[&str]) -> Result<GitResult, GitError> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!("Running: git {}", args.join(" "));

        let output = cmd.output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        if !success {
            error!("Git command failed: {}", stderr);
        }

        Ok(GitResult {
            success,
            stdout,
            stderr,
        })
    }

    /// Get current status
    pub async fn status(&self) -> Result<String, GitError> {
        let result = self.run_git(&["status", "--porcelain"]).await?;
        if result.success {
            Ok(result.stdout)
        } else {
            Err(GitError::OperationFailed {
                operation: "get status".to_string(),
                details: result.stderr,
            })
        }
    }

    /// Check if working directory is clean
    pub async fn is_clean(&self) -> Result<bool, GitError> {
        let status = self.status().await?;
        Ok(status.trim().is_empty())
    }

    /// Get current branch name
    pub async fn current_branch(&self) -> Result<String, GitError> {
        let symbolic = self
            .run_git(&["symbolic-ref", "--short", "-q", "HEAD"])
            .await?;
        if symbolic.success {
            let name = symbolic.stdout.trim().to_string();
            if !name.is_empty() {
                return Ok(name);
            }
        }

        let result = self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"]).await?;
        if result.success {
            Ok(result.stdout.trim().to_string())
        } else {
            Err(GitError::OperationFailed {
                operation: "get branch".to_string(),
                details: result.stderr,
            })
        }
    }

    /// Get current commit hash
    pub async fn current_commit(&self) -> Result<String, GitError> {
        let result = self.run_git(&["rev-parse", "HEAD"]).await?;
        if result.success {
            Ok(result.stdout.trim().to_string())
        } else {
            Err(GitError::OperationFailed {
                operation: "get commit".to_string(),
                details: result.stderr,
            })
        }
    }

    /// Get remote URL
    pub async fn remote_url(&self, remote: &str) -> Result<Option<String>, GitError> {
        let result = self.run_git(&["remote", "get-url", remote]).await?;
        if result.success {
            Ok(Some(result.stdout.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    /// List branches
    pub async fn list_branches(&self) -> Result<Vec<BranchInfo>, GitError> {
        let result = self.run_git(&["branch", "-vv"]).await?;
        if !result.success {
            return Err(GitError::OperationFailed {
                operation: "list branches".to_string(),
                details: result.stderr,
            });
        }

        let mut branches = Vec::new();
        for line in result.stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let is_current = line.starts_with('*');
            let name = line.split_whitespace().nth(1).unwrap_or("").to_string();

            // Parse upstream if present
            let upstream = match (line.find('['), line.find(']')) {
                (Some(start), Some(end)) if start < end => Some(line[start + 1..end].to_string()),
                _ => None,
            };

            branches.push(BranchInfo {
                name,
                is_current,
                upstream,
            });
        }

        Ok(branches)
    }

    /// Create a new branch
    pub async fn create_branch(&self, name: &str) -> Result<(), GitError> {
        let result = self.run_git(&["checkout", "-b", name]).await?;
        if result.success {
            info!("Created and checked out branch: {}", name);
            Ok(())
        } else {
            Err(GitError::OperationFailed {
                operation: "create branch".to_string(),
                details: result.stderr,
            })
        }
    }

    /// Apply a patch (dry-run first if not in dry-run mode)
    pub async fn apply_patch(&self, patch: &str, dry_run: bool) -> Result<(), GitError> {
        if dry_run {
            // Only do dry-run check
            let output = self
                .run_git_with_stdin(&["apply", "--check", "-"], patch)
                .await?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                return Err(GitError::OperationFailed {
                    operation: "patch dry-run".to_string(),
                    details: stderr,
                });
            }
            info!("Patch dry-run successful");
        } else {
            // First do dry-run check
            let check_output = self
                .run_git_with_stdin(&["apply", "--check", "-"], patch)
                .await?;
            if !check_output.status.success() {
                let stderr = String::from_utf8_lossy(&check_output.stderr).to_string();
                return Err(GitError::OperationFailed {
                    operation: "patch check".to_string(),
                    details: stderr,
                });
            }

            // Then apply for real
            let output = self.run_git_with_stdin(&["apply", "-"], patch).await?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                return Err(GitError::OperationFailed {
                    operation: "apply patch".to_string(),
                    details: stderr,
                });
            }
            info!("Patch applied successfully");
        }

        Ok(())
    }

    /// Run a git command with input from stdin
    async fn run_git_with_stdin(
        &self,
        args: &[&str],
        stdin_input: &str,
    ) -> Result<std::process::Output, GitError> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        use tokio::io::AsyncWriteExt;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(stdin_input.as_bytes()).await?;
        }

        Ok(child.wait_with_output().await?)
    }

    /// Stage files
    pub async fn add(&self, paths: &[&str]) -> Result<(), GitError> {
        let mut args = vec!["add"];
        args.extend(paths);

        let result = self.run_git(&args).await?;
        if result.success {
            Ok(())
        } else {
            Err(GitError::OperationFailed {
                operation: "stage files".to_string(),
                details: result.stderr,
            })
        }
    }

    /// Commit changes
    pub async fn commit(&self, message: &str) -> Result<(), GitError> {
        let result = self.run_git(&["commit", "-m", message]).await?;
        if result.success {
            info!("Created commit: {}", message.lines().next().unwrap_or(""));
            Ok(())
        } else {
            Err(GitError::OperationFailed {
                operation: "commit".to_string(),
                details: result.stderr,
            })
        }
    }

    /// Get diff
    pub async fn diff(&self, target: Option<&str>) -> Result<String, GitError> {
        let args = match target {
            Some(t) => vec!["diff", t],
            None => vec!["diff"],
        };

        let result = self.run_git(&args).await?;
        if result.success {
            Ok(result.stdout)
        } else {
            Err(GitError::OperationFailed {
                operation: "get diff".to_string(),
                details: result.stderr,
            })
        }
    }

    /// Clone a repository
    pub async fn clone(url: &str, target: impl AsRef<Path>) -> Result<Self, GitError> {
        let target = target.as_ref();

        info!("Cloning {} into {}", url, target.display());

        let result = Command::new("git")
            .args(["clone", url, &target.to_string_lossy()])
            .output()
            .await?;

        if result.status.success() {
            info!("Clone successful");
            Self::open(target).await
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            Err(GitError::CloneFailed(stderr))
        }
    }
}

/// Clone a repository from URL
///
/// This is a convenience wrapper around `GitRepo::clone`.
pub async fn clone_repo(url: &str, target: impl AsRef<Path>) -> Result<GitRepo, GitError> {
    GitRepo::clone(url, target).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_is_repo() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        assert!(!GitRepo::is_repo(temp.path()).await);

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(temp.path())
            .output()
            .await?;

        assert!(GitRepo::is_repo(temp.path()).await);
        Ok(())
    }

    #[tokio::test]
    async fn test_current_branch() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let init = Command::new("git")
            .args(["init"])
            .current_dir(temp.path())
            .output()
            .await?;
        if !init.status.success() {
            return Err(anyhow::anyhow!(
                "git init failed: {}",
                String::from_utf8_lossy(&init.stderr)
            ));
        }

        // Configure git for the test
        let config_email = Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp.path())
            .output()
            .await?;
        if !config_email.status.success() {
            return Err(anyhow::anyhow!(
                "git config user.email failed: {}",
                String::from_utf8_lossy(&config_email.stderr)
            ));
        }

        let config_name = Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(temp.path())
            .output()
            .await?;
        if !config_name.status.success() {
            return Err(anyhow::anyhow!(
                "git config user.name failed: {}",
                String::from_utf8_lossy(&config_name.stderr)
            ));
        }

        tokio::fs::write(temp.path().join("README.md"), "# test\n").await?;

        let add = Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .await?;
        if !add.status.success() {
            return Err(anyhow::anyhow!(
                "git add failed: {}",
                String::from_utf8_lossy(&add.stderr)
            ));
        }

        // Create initial commit so HEAD exists
        let commit = Command::new("git")
            .args(["-c", "commit.gpgsign=false", "commit", "-m", "Initial commit"])
            .current_dir(temp.path())
            .output()
            .await?;
        if !commit.status.success() {
            return Err(anyhow::anyhow!(
                "git commit failed: {}",
                String::from_utf8_lossy(&commit.stderr)
            ));
        }

        let repo = GitRepo::open(temp.path()).await?;
        let branch = repo.current_branch().await?;
        assert!(!branch.is_empty());
        Ok(())
    }
}
