export interface CreateProjectRequest {
  name: string
  pg_version?: string
}

export interface UpdateProjectRequest {
  name?: string
  gc_period?: string
  gc_horizon?: number
  pitr_interval?: string
  checkpoint_distance?: number
  checkpoint_timeout?: string
}
