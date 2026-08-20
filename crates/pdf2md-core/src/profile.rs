use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExecutionProfile {
    /// Maximum multi-threaded throughput. Uses bounded Rayon threadpool and in-memory caching.
    Fast,

    /// Balanced trade-off between throughput and memory consumption (Default).
    #[default]
    Balanced,

    /// Minimum memory footprint. Page-by-page streaming, immediate buffer deallocation.
    /// Can process 1,000+ page documents on 128MB VPS servers.
    LowMemory,
}

impl ExecutionProfile {
    pub fn max_concurrency(&self, available_cores: usize) -> usize {
        match self {
            Self::Fast => available_cores.max(1),
            Self::Balanced => (available_cores / 2).clamp(1, 8),
            Self::LowMemory => 1, // Single-threaded page-by-page stream
        }
    }

    pub fn target_memory_budget_mb(&self) -> usize {
        match self {
            Self::Fast => 1024,
            Self::Balanced => 256,
            Self::LowMemory => 32,
        }
    }

    pub fn enable_caching(&self) -> bool {
        match self {
            Self::Fast | Self::Balanced => true,
            Self::LowMemory => false,
        }
    }
}
