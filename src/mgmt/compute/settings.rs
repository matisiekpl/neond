use serde::Serialize;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::model::branch::Branch;

pub const DEFAULT_SHARED_BUFFERS_MB: i32 = 128;
pub const MIN_SHARED_BUFFERS_MB: i32 = 64;
pub const DEFAULT_MAX_CONNECTIONS: i32 = 100;
const MAX_MAINTENANCE_WORK_MEM_MB: i32 = 2048;
const MAX_DERIVED_WORK_MEM_MB: i32 = 64;

pub struct HostMemory {
    pub total_mb: i64,
    pub available_mb: i64,
}

pub fn host_memory() -> HostMemory {
    let system = System::new_with_specifics(
        RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()),
    );
    HostMemory {
        total_mb: (system.total_memory() / (1024 * 1024)) as i64,
        available_mb: (system.available_memory() / (1024 * 1024)) as i64,
    }
}

/// Postgres' usual 25%-of-RAM guidance, applied to the sum over all running computes.
pub fn shared_buffers_budget_mb(host_total_mb: i64) -> i64 {
    host_total_mb / 4
}

pub fn recommend_shared_buffers_mb(
    host_total_mb: i64,
    other_running_shared_buffers_mb: i64,
    max_shared_buffers_mb: i32,
) -> i32 {
    let remaining = shared_buffers_budget_mb(host_total_mb) - other_running_shared_buffers_mb;
    let mut recommended = MIN_SHARED_BUFFERS_MB as i64;
    while recommended * 2 <= remaining {
        recommended *= 2;
    }
    recommended.min(max_shared_buffers_mb as i64).max(MIN_SHARED_BUFFERS_MB as i64) as i32
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComputeSettings {
    pub shared_buffers_mb: i32,
    pub effective_cache_size_mb: i32,
    pub maintenance_work_mem_mb: i32,
    pub work_mem_mb: i32,
    pub max_connections: i32,
}

#[derive(Serialize, Clone, Copy, Debug, Default)]
pub struct ComputeSettingsOverrides {
    pub effective_cache_size_mb: Option<i32>,
    pub maintenance_work_mem_mb: Option<i32>,
    pub work_mem_mb: Option<i32>,
    pub max_connections: Option<i32>,
}

impl ComputeSettingsOverrides {
    pub fn from_branch(branch: &Branch) -> Self {
        Self {
            effective_cache_size_mb: branch.effective_cache_size_mb,
            maintenance_work_mem_mb: branch.maintenance_work_mem_mb,
            work_mem_mb: branch.work_mem_mb,
            max_connections: branch.max_connections,
        }
    }
}

impl ComputeSettings {
    pub fn resolve(branch: &Branch) -> Self {
        Self::derive(branch.shared_buffers_mb, ComputeSettingsOverrides::from_branch(branch))
    }

    pub fn derive(shared_buffers_mb: i32, overrides: ComputeSettingsOverrides) -> Self {
        Self {
            shared_buffers_mb,
            effective_cache_size_mb: overrides
                .effective_cache_size_mb
                .unwrap_or(shared_buffers_mb.saturating_mul(4)),
            maintenance_work_mem_mb: overrides
                .maintenance_work_mem_mb
                .unwrap_or(shared_buffers_mb.min(MAX_MAINTENANCE_WORK_MEM_MB)),
            work_mem_mb: overrides
                .work_mem_mb
                .unwrap_or((shared_buffers_mb / 16).clamp(1, MAX_DERIVED_WORK_MEM_MB)),
            max_connections: overrides.max_connections.unwrap_or(DEFAULT_MAX_CONNECTIONS),
        }
    }

    pub fn validate(&self, max_shared_buffers_mb: i32) -> Result<()> {
        let invalid = |reason: String| Err(AppError::ComputeSettingsInvalid { reason });
        if self.shared_buffers_mb < MIN_SHARED_BUFFERS_MB || self.shared_buffers_mb > max_shared_buffers_mb {
            return invalid(format!(
                "shared_buffers must be between {}MB and {}MB",
                MIN_SHARED_BUFFERS_MB, max_shared_buffers_mb
            ));
        }
        if self.effective_cache_size_mb < self.shared_buffers_mb {
            return invalid("effective_cache_size must not be smaller than shared_buffers".to_string());
        }
        if self.maintenance_work_mem_mb < 1 {
            return invalid("maintenance_work_mem must be at least 1MB".to_string());
        }
        if self.work_mem_mb < 1 {
            return invalid("work_mem must be at least 1MB".to_string());
        }
        if !(10..=1000).contains(&self.max_connections) {
            return invalid("max_connections must be between 10 and 1000".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_previous_hardcoded_config() {
        let settings = ComputeSettings::derive(DEFAULT_SHARED_BUFFERS_MB, ComputeSettingsOverrides::default());
        assert_eq!(
            settings,
            ComputeSettings {
                shared_buffers_mb: 128,
                effective_cache_size_mb: 512,
                maintenance_work_mem_mb: 128,
                work_mem_mb: 8,
                max_connections: 100,
            }
        );
    }

    #[test]
    fn derived_values_are_capped() {
        let settings = ComputeSettings::derive(8192, ComputeSettingsOverrides::default());
        assert_eq!(settings.effective_cache_size_mb, 32768);
        assert_eq!(settings.maintenance_work_mem_mb, 2048);
        assert_eq!(settings.work_mem_mb, 64);
    }

    #[test]
    fn overrides_win_over_derivation() {
        let overrides = ComputeSettingsOverrides {
            effective_cache_size_mb: Some(1000),
            maintenance_work_mem_mb: Some(10),
            work_mem_mb: Some(128),
            max_connections: Some(300),
        };
        let settings = ComputeSettings::derive(256, overrides);
        assert_eq!(settings.effective_cache_size_mb, 1000);
        assert_eq!(settings.maintenance_work_mem_mb, 10);
        assert_eq!(settings.work_mem_mb, 128);
        assert_eq!(settings.max_connections, 300);
    }

    #[test]
    fn recommendation_fits_remaining_budget() {
        assert_eq!(recommend_shared_buffers_mb(16384, 0, 8192), 4096);
        assert_eq!(recommend_shared_buffers_mb(16384, 3000, 8192), 1024);
        assert_eq!(recommend_shared_buffers_mb(16384, 0, 1024), 1024);
        assert_eq!(recommend_shared_buffers_mb(2048, 4096, 8192), MIN_SHARED_BUFFERS_MB);
    }

    #[test]
    fn validation_rejects_out_of_range_values() {
        let defaults = ComputeSettingsOverrides::default();
        assert!(ComputeSettings::derive(32, defaults).validate(1024).is_err());
        assert!(ComputeSettings::derive(2048, defaults).validate(1024).is_err());
        assert!(ComputeSettings::derive(256, defaults).validate(1024).is_ok());
        let small_cache = ComputeSettingsOverrides { effective_cache_size_mb: Some(100), ..defaults };
        assert!(ComputeSettings::derive(256, small_cache).validate(1024).is_err());
    }
}
