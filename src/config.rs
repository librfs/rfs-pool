// pool/src/config.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

pub fn generate_default_pools_config() -> &'static str {
    r#"# Configuration for rfs storage pools.
# Each pool represents a storage location that rfs will manage.

[[pool]]
pool_id = 1
is_removable = false
path = "/mnt/pool1"

[[pool]]
pool_id = 2
is_removable = false
path = "/mnt/pool2"
"#
}