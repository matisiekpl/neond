export interface ComputeSettingsOverrides {
  effective_cache_size_mb: number | null
  maintenance_work_mem_mb: number | null
  work_mem_mb: number | null
  max_connections: number | null
}

export interface ComputeSettingsValues {
  shared_buffers_mb: number
  effective_cache_size_mb: number
  maintenance_work_mem_mb: number
  work_mem_mb: number
  max_connections: number
}

export interface ComputeSettings {
  shared_buffers_mb: number
  overrides: ComputeSettingsOverrides
  effective: ComputeSettingsValues
  max_shared_buffers_mb: number
  memory: ComputeMemoryOverview
}

export interface ComputeMemoryOverview {
  host_total_mb: number
  host_available_mb: number
  shared_buffers_budget_mb: number
  other_running_computes: number
  other_running_shared_buffers_mb: number
  recommended_shared_buffers_mb: number
}

export interface UpdateComputeSettingsRequest extends ComputeSettingsOverrides {
  shared_buffers_mb: number
}
