// Cerebras-MAKER: Shadow Git - Transactional File System
// PRD Section 5: Reliability Layer - The Shadow Git

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub message: String,
    pub timestamp_ms: u64,
    pub commit_hash: Option<String>,
}

/// Shadow Git - provides transactional file system operations
pub struct ShadowGit {
    workspace_path: PathBuf,
    snapshots: Vec<Snapshot>,
    current_snapshot_idx: Option<usize>,
    repo: Option<gix::Repository>,
}

impl ShadowGit {
    /// Create a new ShadowGit instance
    pub fn new(workspace_path: &str) -> Self {
        let path = PathBuf::from(workspace_path);
        let repo = gix::open(&path).ok();
        
        Self {
            workspace_path: path,
            snapshots: Vec::new(),
            current_snapshot_idx: None,
            repo,
        }
    }

    /// Initialize or open the repository
    pub fn init(&mut self) -> Result<()> {
        if self.repo.is_none() {
            // Try to open existing repo first
            match gix::open(&self.workspace_path) {
                Ok(repo) => {
                    self.repo = Some(repo);
                }
                Err(_) => {
                    // Initialize new repo
                    gix::init(&self.workspace_path)?;
                    self.repo = Some(gix::open(&self.workspace_path)?);
                }
            }
        }
        Ok(())
    }

    /// Create a snapshot of the current state
    /// PRD 5.1: "Before any Rhai script touches disk, gitoxide creates a blob"
    pub fn snapshot(&mut self, message: &str) -> Result<Snapshot> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let id = format!("snap_{}", timestamp);
        
        let commit_hash = if self.repo.is_some() {
            // Stage all changes and create a commit
            self.stage_all()?;
            match self.create_commit(message) {
                Ok(hash) => Some(hash),
                Err(_) => None,
            }
        } else {
            None
        };

        let snapshot = Snapshot {
            id,
            message: message.to_string(),
            timestamp_ms: timestamp,
            commit_hash,
        };

        self.snapshots.push(snapshot.clone());
        self.current_snapshot_idx = Some(self.snapshots.len() - 1);

        Ok(snapshot)
    }

    /// Stage all changes in the workspace
    fn stage_all(&self) -> Result<()> {
        // Use git command as fallback since gix staging is complex
        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["add", "-A"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to stage changes"));
        }
        Ok(())
    }

    /// Create a commit with the given message
    fn create_commit(&self, message: &str) -> Result<String> {
        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["commit", "-m", message, "--allow-empty"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create commit: {}", stderr));
        }

        // Get the commit hash
        let hash_output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["rev-parse", "HEAD"])
            .output()?;
        
        Ok(String::from_utf8_lossy(&hash_output.stdout).trim().to_string())
    }

    /// Rollback to the previous snapshot
    /// PRD 5.1: "gitoxide reverts the index to the snapshot instantly"
    pub fn rollback(&mut self) -> Result<()> {
        if self.snapshots.len() < 2 {
            return Err(anyhow!("No previous snapshot to rollback to"));
        }

        // Get the previous snapshot
        let prev_idx = self.snapshots.len() - 2;
        let prev_snapshot = &self.snapshots[prev_idx];

        if let Some(ref hash) = prev_snapshot.commit_hash {
            let output = std::process::Command::new("git")
                .current_dir(&self.workspace_path)
                .args(["reset", "--hard", hash])
                .output()?;

            if !output.status.success() {
                return Err(anyhow!("Failed to rollback"));
            }
        }

        // Remove the current snapshot
        self.snapshots.pop();
        self.current_snapshot_idx = Some(prev_idx);

        Ok(())
    }

    /// Rollback to a specific snapshot by ID
    pub fn rollback_to(&mut self, snapshot_id: &str) -> Result<()> {
        let idx = self.snapshots.iter().position(|s| s.id == snapshot_id)
            .ok_or_else(|| anyhow!("Snapshot not found: {}", snapshot_id))?;

        let snapshot = &self.snapshots[idx];

        if let Some(ref hash) = snapshot.commit_hash {
            let output = std::process::Command::new("git")
                .current_dir(&self.workspace_path)
                .args(["reset", "--hard", hash])
                .output()?;

            if !output.status.success() {
                return Err(anyhow!("Failed to rollback to snapshot"));
            }
        }

        // Truncate snapshots after this point
        self.snapshots.truncate(idx + 1);
        self.current_snapshot_idx = Some(idx);

        Ok(())
    }

    /// Get all snapshots
    pub fn get_snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }

    /// Get the current snapshot
    pub fn current_snapshot(&self) -> Option<&Snapshot> {
        self.current_snapshot_idx.and_then(|idx| self.snapshots.get(idx))
    }

    /// Squash all snapshots into a single commit
    /// PRD 5.1: "Only when PLAN.md is marked Complete does Shadow Repo squash"
    pub fn squash(&mut self, final_message: &str) -> Result<String> {
        if self.snapshots.is_empty() {
            return Err(anyhow!("No snapshots to squash"));
        }

        let first_snapshot = &self.snapshots[0];

        // Get the parent of the first snapshot
        let parent_output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["rev-parse", &format!("{}^", first_snapshot.commit_hash.as_deref().unwrap_or("HEAD"))])
            .output()?;

        let parent_hash = if parent_output.status.success() {
            String::from_utf8_lossy(&parent_output.stdout).trim().to_string()
        } else {
            // If no parent, use empty tree
            "".to_string()
        };

        // Soft reset to parent
        if !parent_hash.is_empty() {
            let _ = std::process::Command::new("git")
                .current_dir(&self.workspace_path)
                .args(["reset", "--soft", &parent_hash])
                .output()?;
        }

        // Create the squashed commit
        self.stage_all()?;
        let hash = self.create_commit(final_message)?;

        // Clear snapshots
        self.snapshots.clear();
        self.current_snapshot_idx = None;

        Ok(hash)
    }

    /// Get the git history for time-travel visualization
    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["log", "--oneline", "-n", &limit.to_string()])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get git history"));
        }

        let entries = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                HistoryEntry {
                    hash: parts.first().unwrap_or(&"").to_string(),
                    message: parts.get(1).unwrap_or(&"").to_string(),
                }
            })
            .collect();

        Ok(entries)
    }
}

/// A git history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub hash: String,
    pub message: String,
}

