// pool/src/mount.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use crate::config;
use once_cell::sync::Lazy;
use rfs_utils::{log, LogLevel};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct PoolsFile {
    pool: Vec<Pool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pool {
    pub pool_id: u64,
    pub is_removable: bool,
    pub path: String,
}

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Pool configuration cannot be empty. Please define at least one pool in pool.toml.")]
    EmptyPools,
    #[error("Pool IDs are not unique or not sequential. IDs must be 1, 2, 3...")]
    InvalidIdSequence,
    #[error("Default pool config created at '{0}'. Please review and configure it before restarting.")]
    MustConfigure(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

pub static POOLS: Lazy<Mutex<Vec<Pool>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub async fn load_and_mount_pools(path_str: &str) -> Result<(), PoolError> {
    let path = Path::new(path_str);

    if !path.exists() {
        log(
            LogLevel::Warn,
            &format!("Pool config not found at {}. Creating default.", path_str),
        );
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, config::generate_default_pools_config())?;
        // Instead of continuing, return a specific error to force the user to configure the file.
        return Err(PoolError::MustConfigure(path_str.to_string()));
    }

    log(LogLevel::Info, &format!("Loading pools from {}", path_str));
    let content = fs::read_to_string(path)?;
    // The `mut` keyword has been removed here as requested.
    let pools_from_file: PoolsFile = toml::from_str(&content)?;
    let mut pools = pools_from_file.pool;

    if pools.is_empty() {
        return Err(PoolError::EmptyPools);
    }

    pools.sort_by_key(|p| p.pool_id);
    for (index, pool) in pools.iter().enumerate() {
        if pool.pool_id != (index as u64) + 1 {
            return Err(PoolError::InvalidIdSequence);
        }
    }
    log(LogLevel::Debug, "Pool IDs are sequential and unique.");

    for pool in &pools {
        if !pool.is_removable {
            match tokio::fs::metadata(&pool.path).await {
                Ok(meta) => {
                    if !meta.is_dir() {
                        log(
                            LogLevel::Warn,
                            &format!(
                                "Path for pool {} ({}) exists but is not a directory.",
                                pool.pool_id, pool.path
                            ),
                        );
                    }
                }
                Err(_) => {
                    log(
                        LogLevel::Warn,
                        &format!("Path for pool {} ({}) is not reachable.", pool.pool_id, pool.path),
                    );
                }
            }
        }
    }
    log(LogLevel::Debug, "Pool path accessibility check complete.");

    let mut pools_guard = POOLS.lock().unwrap();
    *pools_guard = pools;

    log(LogLevel::Info, "Storage pools mounted successfully.");
    Ok(())
}