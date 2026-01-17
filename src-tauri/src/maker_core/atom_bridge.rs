//! Atom Bridge - Safe Async-to-Sync Bridging for Rhai Runtime
//!
//! This module provides a dedicated worker thread pool that handles async operations
//! from the synchronous Rhai scripting engine without blocking the async runtime
//! or creating new runtimes per call.
//!
//! Architecture:
//! ```text
//! ┌──────────────────┐     mpsc::channel      ┌──────────────────────┐
//! │   Rhai Runtime   │ ─────────────────────> │   AtomWorkerPool     │
//! │   (sync calls)   │                        │  (dedicated tokio    │
//! │                  │ <───────────────────── │   runtime thread)    │
//! └──────────────────┘     oneshot::channel   └──────────────────────┘
//! ```

use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use once_cell::sync::Lazy;

use crate::agents::atom_executor::{AtomExecutor, AtomInput};
use crate::llm::LlmConfig;
use crate::maker_core::AtomResult;

/// Atom-specific request for executing atoms
pub struct AtomRequest {
    pub input: AtomInput,
    pub response_tx: oneshot::Sender<Result<AtomResult, String>>,
}

/// Worker pool handle for sending requests
pub struct AtomWorkerPool {
    atom_tx: mpsc::Sender<AtomRequest>,
    executor: Arc<AtomExecutor>,
}

impl AtomWorkerPool {
    /// Create a new worker pool with dedicated runtime thread
    pub fn new(llm_config: Arc<LlmConfig>) -> Self {
        let (atom_tx, mut atom_rx) = mpsc::channel::<AtomRequest>(64);

        let executor = Arc::new(AtomExecutor::new((*llm_config).clone()));
        let worker_executor = executor.clone();

        // Spawn dedicated runtime thread that lives for application lifetime
        std::thread::Builder::new()
            .name("atom-worker-pool".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(4)
                    .thread_name("atom-worker")
                    .enable_all()
                    .build()
                    .expect("Failed to create atom worker runtime");

                rt.block_on(async move {
                    while let Some(request) = atom_rx.recv().await {
                        let exec = worker_executor.clone();
                        tokio::spawn(async move {
                            let result = exec.execute(request.input).await;
                            let _ = request.response_tx.send(result);
                        });
                    }
                });
            })
            .expect("Failed to spawn atom worker thread");

        Self {
            atom_tx,
            executor,
        }
    }
    
    /// Execute an atom synchronously (safe to call from Rhai)
    pub fn execute_atom_sync(&self, input: AtomInput) -> Result<AtomResult, String> {
        let (response_tx, response_rx) = oneshot::channel();
        
        // Send to worker pool (non-blocking send, but we block on receive)
        self.atom_tx.blocking_send(AtomRequest { input, response_tx })
            .map_err(|e| format!("Failed to send to worker pool: {}", e))?;
        
        // Wait for response (blocking, but on the worker pool's dedicated thread)
        response_rx.blocking_recv()
            .map_err(|_| "Worker dropped response channel".to_string())?
    }
    
    /// Get the underlying executor for direct async use
    pub fn executor(&self) -> Arc<AtomExecutor> {
        self.executor.clone()
    }
}

/// Global worker pool instance
static ATOM_POOL: Lazy<Mutex<Option<AtomWorkerPool>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the global atom worker pool
pub fn init_atom_pool(llm_config: Arc<LlmConfig>) {
    let mut pool = ATOM_POOL.lock().unwrap();
    if pool.is_none() {
        *pool = Some(AtomWorkerPool::new(llm_config));
    }
}

/// Check if the pool is initialized
pub fn is_pool_initialized() -> bool {
    ATOM_POOL.lock().map(|p| p.is_some()).unwrap_or(false)
}

/// Execute an atom synchronously using the global pool
pub fn execute_atom_sync(input: AtomInput) -> Result<AtomResult, String> {
    let pool = ATOM_POOL.lock()
        .map_err(|_| "Failed to acquire atom pool lock")?;
    
    pool.as_ref()
        .ok_or_else(|| "Atom pool not initialized. Call init_atom_pool first.".to_string())?
        .execute_atom_sync(input)
}

/// Get the executor for direct async operations
pub fn get_executor() -> Result<Arc<AtomExecutor>, String> {
    let pool = ATOM_POOL.lock()
        .map_err(|_| "Failed to acquire atom pool lock")?;
    
    pool.as_ref()
        .ok_or_else(|| "Atom pool not initialized".to_string())
        .map(|p| p.executor())
}

