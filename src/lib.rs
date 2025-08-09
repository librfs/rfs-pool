// pool/src/lib.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

mod config;
mod mount;

pub use mount::{load_and_mount_pools, Pool, POOLS, PoolError};