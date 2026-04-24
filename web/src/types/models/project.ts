export interface Project {
  id: string
  organization_id: string
  name: string
  pg_version: string
  created_at: string
  updated_at: string
  gc_period?: string
  gc_horizon?: number
  pitr_interval?: string
  checkpoint_distance?: number
  checkpoint_timeout?: string
  size?: number
}
