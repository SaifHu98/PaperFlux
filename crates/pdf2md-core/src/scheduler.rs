use crate::profile::ExecutionProfile;

pub struct AdaptiveScheduler {
    profile: ExecutionProfile,
    available_cores: usize,
    memory_budget_mb: usize,
}

#[derive(Debug, Clone)]
pub struct SchedulePlan {
    pub concurrency: usize,
    pub chunk_size: usize,
    pub use_cache: bool,
    pub stream_pages: bool,
}

impl AdaptiveScheduler {
    pub fn new(profile: ExecutionProfile, custom_memory_budget_mb: Option<usize>) -> Self {
        let cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        let memory_budget_mb =
            custom_memory_budget_mb.unwrap_or_else(|| profile.target_memory_budget_mb());

        Self {
            profile,
            available_cores: cores,
            memory_budget_mb,
        }
    }

    pub fn plan(&self, total_pages: usize, total_file_size_bytes: usize) -> SchedulePlan {
        let est_page_size_mb =
            (total_file_size_bytes as f32 / (total_pages.max(1) as f32 * 1024.0 * 1024.0)).max(0.5);
        let max_concurrent_by_mem =
            ((self.memory_budget_mb as f32) / (est_page_size_mb * 4.0)).floor() as usize;

        let max_profile_concurrency = self.profile.max_concurrency(self.available_cores);
        let concurrency = max_profile_concurrency
            .min(max_concurrent_by_mem.max(1))
            .min(total_pages.max(1));

        let stream_pages = self.profile == ExecutionProfile::LowMemory || total_pages > 500;
        let chunk_size = if stream_pages {
            1
        } else {
            (total_pages / concurrency.max(1)).clamp(1, 10)
        };

        SchedulePlan {
            concurrency,
            chunk_size,
            use_cache: self.profile.enable_caching(),
            stream_pages,
        }
    }
}
