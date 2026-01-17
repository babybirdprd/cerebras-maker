// Cerebras-MAKER: Shadow Git - Transactional File System
// PRD Section 5: Reliability Layer - The Shadow Git
// Uses native gitoxide (gix) for read operations, with fallback for complex writes

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of snapshots to prevent unbounded memory growth
const MAX_SNAPSHOTS: usize = 100;

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

/// HIGH-14: Snapshot state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotState {
    snapshots: Vec<Snapshot>,
    current_snapshot_idx: Option<usize>,
}

impl ShadowGit {
    /// Create a new ShadowGit instance
    /// HIGH-14: Automatically loads persisted snapshots if available
    pub fn new(workspace_path: &str) -> Self {
        let path = PathBuf::from(workspace_path);
        let repo = gix::open(&path).ok();

        // HIGH-14: Try to load persisted snapshot state
        let (snapshots, current_snapshot_idx) = Self::load_snapshot_state_from_path(&path)
            .map(|state| (state.snapshots, state.current_snapshot_idx))
            .unwrap_or_else(|_| (Vec::new(), None));

        Self {
            workspace_path: path,
            snapshots,
            current_snapshot_idx,
            repo,
        }
    }

    /// HIGH-14: Get the path to the snapshot state file
    fn snapshot_state_path(&self) -> PathBuf {
        self.workspace_path.join(".maker").join("snapshots.json")
    }

    /// HIGH-14: Load snapshot state from a path
    fn load_snapshot_state_from_path(workspace_path: &PathBuf) -> Result<SnapshotState> {
        let state_path = workspace_path.join(".maker").join("snapshots.json");
        let content = std::fs::read_to_string(&state_path)?;
        let state: SnapshotState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// HIGH-14: Persist snapshot state to disk
    pub fn persist_snapshots(&self) -> Result<()> {
        let state = SnapshotState {
            snapshots: self.snapshots.clone(),
            current_snapshot_idx: self.current_snapshot_idx,
        };

        let state_path = self.snapshot_state_path();

        // Ensure .maker directory exists
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&state)?;
        std::fs::write(&state_path, content)?;

        Ok(())
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
    /// HIGH-14: Now persists snapshot state after creation
    /// Enforces MAX_SNAPSHOTS limit by removing oldest snapshots when exceeded
    pub fn snapshot(&mut self, message: &str) -> Result<Snapshot> {
        // Enforce max snapshots limit by removing oldest when limit is reached
        while self.snapshots.len() >= MAX_SNAPSHOTS {
            self.snapshots.remove(0);
            // Adjust current_snapshot_idx after removal
            if let Some(idx) = self.current_snapshot_idx {
                if idx > 0 {
                    self.current_snapshot_idx = Some(idx - 1);
                } else {
                    self.current_snapshot_idx = None;
                }
            }
        }

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

        // HIGH-14: Persist snapshot state to disk
        if let Err(e) = self.persist_snapshots() {
            eprintln!("Warning: Failed to persist snapshot state: {}", e);
        }

        Ok(snapshot)
    }

    /// Stage all changes in the workspace
    /// Uses git command for reliable cross-platform staging
    #[cfg(not(feature = "native-git"))]
    fn stage_all(&self) -> Result<()> {
        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["add", "-A"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to stage changes"));
        }
        Ok(())
    }

    /// Stage all changes using native gix
    #[cfg(feature = "native-git")]
    fn stage_all(&self) -> Result<()> {
        self.stage_all_native()
            .map_err(|e| anyhow!("{}", e))
    }

    /// Create a commit with the given message
    #[cfg(not(feature = "native-git"))]
    fn create_commit(&self, message: &str) -> Result<String> {
        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["commit", "-m", message, "--allow-empty"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create commit: {}", stderr));
        }

        // Get the commit hash using gix for efficiency
        if let Some(ref repo) = self.repo {
            if let Ok(head) = repo.head_commit() {
                return Ok(head.id().to_string());
            }
        }

        // Fallback to git command
        let hash_output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["rev-parse", "HEAD"])
            .output()?;

        Ok(String::from_utf8_lossy(&hash_output.stdout).trim().to_string())
    }

    /// Create a commit using native gix
    #[cfg(feature = "native-git")]
    fn create_commit(&self, message: &str) -> Result<String> {
        self.create_commit_native(message)
            .map_err(|e| anyhow!("{}", e))
    }

    /// Rollback to the previous snapshot
    /// PRD 5.1: "gitoxide reverts the index to the snapshot instantly"
    /// HIGH-12: Added workspace verification before rollback
    pub fn rollback(&mut self) -> Result<()> {
        if self.snapshots.len() < 2 {
            return Err(anyhow!("No previous snapshot to rollback to"));
        }

        // HIGH-12: Verify workspace still exists and is accessible
        if !self.workspace_path.exists() {
            return Err(anyhow!("Workspace path no longer exists: {:?}", self.workspace_path));
        }
        if !self.workspace_path.is_dir() {
            return Err(anyhow!("Workspace path is not a directory: {:?}", self.workspace_path));
        }

        // Get the previous snapshot
        let prev_idx = self.snapshots.len() - 2;
        let prev_snapshot = &self.snapshots[prev_idx];

        if let Some(ref hash) = prev_snapshot.commit_hash {
            self.reset_hard(hash)?;
        }

        // Remove the current snapshot
        self.snapshots.pop();
        self.current_snapshot_idx = Some(prev_idx);

        // HIGH-14: Persist snapshot state after rollback
        if let Err(e) = self.persist_snapshots() {
            eprintln!("Warning: Failed to persist snapshot state after rollback: {}", e);
        }

        Ok(())
    }

    /// Rollback to a specific snapshot by ID
    /// HIGH-12: Added workspace verification before rollback
    pub fn rollback_to(&mut self, snapshot_id: &str) -> Result<()> {
        // HIGH-12: Verify workspace still exists and is accessible
        if !self.workspace_path.exists() {
            return Err(anyhow!("Workspace path no longer exists: {:?}", self.workspace_path));
        }
        if !self.workspace_path.is_dir() {
            return Err(anyhow!("Workspace path is not a directory: {:?}", self.workspace_path));
        }

        let idx = self.snapshots.iter().position(|s| s.id == snapshot_id)
            .ok_or_else(|| anyhow!("Snapshot not found: {}", snapshot_id))?;

        let snapshot = &self.snapshots[idx];

        if let Some(ref hash) = snapshot.commit_hash {
            self.reset_hard(hash)?;
        }

        // Truncate snapshots after this point
        self.snapshots.truncate(idx + 1);
        self.current_snapshot_idx = Some(idx);

        // HIGH-14: Persist snapshot state after rollback
        if let Err(e) = self.persist_snapshots() {
            eprintln!("Warning: Failed to persist snapshot state after rollback: {}", e);
        }

        Ok(())
    }

    /// Perform a hard reset to a specific commit
    /// Uses git command for reliable reset with working directory update
    #[cfg(not(feature = "native-git"))]
    fn reset_hard(&self, commit_hash: &str) -> Result<()> {
        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_path)
            .args(["reset", "--hard", commit_hash])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to reset to {}", commit_hash));
        }
        Ok(())
    }

    /// Perform a hard reset to a specific commit using native gix
    #[cfg(feature = "native-git")]
    fn reset_hard(&self, commit_hash: &str) -> Result<()> {
        self.reset_hard_native(commit_hash)
            .map_err(|e| anyhow!("{}", e))
    }

    /// Stage all changes using native gix index API
    /// Walks the worktree and adds all files to the index
    #[cfg(feature = "native-git")]
    pub fn stage_all_native(&self) -> Result<(), String> {
        let repo = self.repo.as_ref().ok_or("Repository not initialized")?;

        // Get the worktree path
        let workdir = repo.workdir().ok_or("No worktree available")?;

        // Create a new empty index state
        let mut index_state = gix::index::State::new(repo.object_hash());

        // Walk the worktree and add all files
        let walker = walkdir::WalkDir::new(workdir)
            .into_iter()
            .filter_entry(|e| {
                // Skip .git directory
                !e.path().components().any(|c| c.as_os_str() == ".git")
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                let rel_path = path.strip_prefix(workdir)
                    .map_err(|e| e.to_string())?;

                // Read file content and compute hash
                let content = std::fs::read(path).map_err(|e| e.to_string())?;
                let blob_id = repo.write_blob(&content)
                    .map_err(|e| e.to_string())?
                    .detach();

                // Create a default stat (we don't need precise stat for staging)
                let stat = gix::index::entry::Stat::default();

                // Determine file mode - default to regular file
                #[cfg(unix)]
                let mode = {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
                    if metadata.permissions().mode() & 0o111 != 0 {
                        gix::index::entry::Mode::FILE_EXECUTABLE
                    } else {
                        gix::index::entry::Mode::FILE
                    }
                };
                #[cfg(not(unix))]
                let mode = gix::index::entry::Mode::FILE;

                // Convert path to BStr - use forward slashes for git compatibility
                let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
                let path_bstr = gix::bstr::BStr::new(rel_path_str.as_bytes());

                // Add entry to index
                index_state.dangerously_push_entry(
                    stat,
                    blob_id,
                    gix::index::entry::Flags::empty(),
                    mode,
                    path_bstr,
                );
            }
        }

        // Sort entries to maintain index invariants
        index_state.sort_entries();

        // Write the index back to disk
        let index_path = repo.index_path();
        let mut index_file = gix::index::File::from_state(index_state, index_path);
        index_file.write(gix::index::write::Options::default())
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Create a commit using native gix API
    /// Returns the commit hash as a string
    #[cfg(feature = "native-git")]
    pub fn create_commit_native(&self, message: &str) -> Result<String, String> {
        let repo = self.repo.as_ref().ok_or("Repository not initialized")?;

        // Get the current index
        let index = repo.index().map_err(|e| e.to_string())?;

        // Build a tree from the index entries using the tree editor
        // Start with an empty tree and add all entries from the index
        let empty_tree_id = repo.empty_tree().id().detach();
        let mut tree_editor = repo.edit_tree(empty_tree_id)
            .map_err(|e| format!("Failed to create tree editor: {}", e))?;

        // Add each entry from the index to the tree
        for entry in index.entries() {
            let path = entry.path(&index);
            // Convert index entry mode to tree entry mode
            let tree_mode = entry.mode.to_tree_entry_mode()
                .ok_or_else(|| format!("Invalid mode for entry: {:?}", entry.mode))?;
            tree_editor.upsert(path, tree_mode.kind(), entry.id)
                .map_err(|e| format!("Failed to add entry to tree: {}", e))?;
        }

        // Write the tree and get its ID
        let tree_id = tree_editor.write()
            .map_err(|e| format!("Failed to write tree: {}", e))?
            .detach();

        // Get parent commit (HEAD), if any
        let parents: Vec<gix::ObjectId> = match repo.head_commit() {
            Ok(commit) => vec![commit.id().detach()],
            Err(_) => vec![], // Initial commit, no parents
        };

        // Create the commit
        let commit_id = repo.commit(
            "HEAD",
            message,
            tree_id,
            parents.iter().map(|id| id.as_ref()),
        ).map_err(|e| format!("Failed to create commit: {}", e))?;

        Ok(commit_id.to_string())
    }

    /// Reset HEAD to a specific commit and checkout the tree using native gix
    #[cfg(feature = "native-git")]
    pub fn reset_hard_native(&self, commit: &str) -> Result<(), String> {
        let repo = self.repo.as_ref().ok_or("Repository not initialized")?;

        // Parse the commit reference
        let commit_id = repo.rev_parse_single(commit)
            .map_err(|e| format!("Failed to parse commit '{}': {}", commit, e))?
            .detach();

        // Get the commit object
        let commit_obj = repo.find_commit(commit_id)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        // Get the tree from the commit
        let tree_id = commit_obj.tree_id()
            .map_err(|e| format!("Failed to get tree: {}", e))?;

        // Get worktree path
        let workdir = repo.workdir()
            .ok_or("No worktree available")?
            .to_path_buf();

        // Create index from the tree and write it to disk
        let index = repo.index_from_tree(&tree_id)
            .map_err(|e| format!("Failed to create index from tree: {}", e))?;

        // Write the index to disk
        let index_path = repo.index_path();
        let mut index_file = gix::index::File::from_state(gix::index::State::from(index), index_path);
        index_file.write(gix::index::write::Options::default())
            .map_err(|e| format!("Failed to write index: {}", e))?;

        // Checkout files from the tree to the worktree
        // We iterate through the tree and write each blob to the worktree
        let tree = repo.find_tree(tree_id.detach())
            .map_err(|e| format!("Failed to find tree: {}", e))?;

        Self::checkout_tree_to_worktree(repo, &tree, &workdir, &workdir)?;

        // Update HEAD to point to the commit
        repo.reference(
            "HEAD",
            commit_id,
            gix::refs::transaction::PreviousValue::Any,
            format!("reset: moving to {}", commit),
        ).map_err(|e| format!("Failed to update HEAD: {}", e))?;

        Ok(())
    }

    /// Helper function to recursively checkout a tree to the worktree
    #[cfg(feature = "native-git")]
    fn checkout_tree_to_worktree(
        repo: &gix::Repository,
        tree: &gix::Tree<'_>,
        workdir: &std::path::Path,
        current_path: &std::path::Path,
    ) -> Result<(), String> {
        for entry in tree.iter() {
            let entry = entry.map_err(|e| format!("Failed to iterate tree: {}", e))?;
            let entry_path = current_path.join(entry.filename().to_string());

            match entry.mode().kind() {
                gix::object::tree::EntryKind::Blob | gix::object::tree::EntryKind::BlobExecutable => {
                    // Read the blob and write to file
                    let blob = repo.find_blob(entry.object_id())
                        .map_err(|e| format!("Failed to find blob: {}", e))?;

                    // Ensure parent directory exists
                    if let Some(parent) = entry_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create directory: {}", e))?;
                    }

                    std::fs::write(&entry_path, &*blob.data)
                        .map_err(|e| format!("Failed to write file: {}", e))?;

                    // Set executable permission on Unix
                    #[cfg(unix)]
                    if entry.mode().kind() == gix::object::tree::EntryKind::BlobExecutable {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = std::fs::metadata(&entry_path)
                            .map_err(|e| e.to_string())?
                            .permissions();
                        perms.set_mode(perms.mode() | 0o111);
                        std::fs::set_permissions(&entry_path, perms)
                            .map_err(|e| e.to_string())?;
                    }
                }
                gix::object::tree::EntryKind::Tree => {
                    // Recursively checkout subtree
                    let subtree = repo.find_tree(entry.object_id())
                        .map_err(|e| format!("Failed to find subtree: {}", e))?;
                    Self::checkout_tree_to_worktree(repo, &subtree, workdir, &entry_path)?;
                }
                _ => {
                    // Skip symlinks and other entry types for now
                }
            }
        }
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
    /// HIGH-13: Improved error handling for soft reset failures
    pub fn squash(&mut self, final_message: &str) -> Result<String> {
        if self.snapshots.is_empty() {
            return Err(anyhow!("No snapshots to squash"));
        }

        let first_snapshot = &self.snapshots[0];

        // Get the parent of the first snapshot
        if let Some(ref hash) = first_snapshot.commit_hash {
            // Soft reset to parent of first snapshot
            let parent_output = std::process::Command::new("git")
                .current_dir(&self.workspace_path)
                .args(["rev-parse", &format!("{}^", hash)])
                .output()?;

            if parent_output.status.success() {
                let parent_hash = String::from_utf8_lossy(&parent_output.stdout).trim().to_string();
                // HIGH-13: Check soft reset result and handle failure
                let reset_output = std::process::Command::new("git")
                    .current_dir(&self.workspace_path)
                    .args(["reset", "--soft", &parent_hash])
                    .output()?;

                if !reset_output.status.success() {
                    let stderr = String::from_utf8_lossy(&reset_output.stderr);
                    return Err(anyhow!("Soft reset failed during squash: {}", stderr));
                }
            } else {
                // No parent commit (first commit in repo) - proceed without reset
                eprintln!("Warning: No parent commit found for squash, proceeding with current state");
            }
        }

        // Create the squashed commit
        self.stage_all()?;
        let hash = self.create_commit(final_message)?;

        // Clear snapshots
        self.snapshots.clear();
        self.current_snapshot_idx = None;

        // HIGH-14: Persist snapshot state after squash
        if let Err(e) = self.persist_snapshots() {
            eprintln!("Warning: Failed to persist snapshot state after squash: {}", e);
        }

        Ok(hash)
    }

    /// Get the git history for time-travel visualization
    /// Uses native gix for efficient history traversal
    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        // Try native gix first for efficiency
        if let Some(ref repo) = self.repo {
            if let Ok(head) = repo.head_commit() {
                let mut entries = Vec::new();
                let mut current_id = Some(head.id().detach());
                let mut count = 0;

                while let Some(id) = current_id {
                    if count >= limit {
                        break;
                    }

                    if let Ok(obj) = repo.find_object(id) {
                        if let Ok(commit) = obj.try_into_commit() {
                            let message = commit.message_raw()
                                .map(|m| m.to_string())
                                .unwrap_or_default();
                            let first_line = message.lines().next().unwrap_or("").to_string();
                            let hash_str = id.to_string();
                            let short_hash = if hash_str.len() >= 7 { &hash_str[..7] } else { &hash_str };

                            entries.push(HistoryEntry {
                                hash: short_hash.to_string(),
                                message: first_line,
                            });

                            current_id = commit.parent_ids().next().map(|p| p.detach());
                            count += 1;
                            continue;
                        }
                    }
                    break;
                }

                if !entries.is_empty() {
                    return Ok(entries);
                }
            }
        }

        // Fallback to git command
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

