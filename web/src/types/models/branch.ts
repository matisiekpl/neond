export type BranchStatus = 'starting' | 'running' | 'stopping' | 'stopped' | 'failed'

export interface Branch {
  id: string
  project_id: string
  name: string
  slug: string
  parent_branch_id: string | null
  ancestor_timeline_id: string | null
  ancestor_lsn: string | null
  endpoint_status: BranchStatus
  connection_string: string | null
  current_logical_size: number
  remote_consistent_lsn_visible: string
  last_record_lsn: string
  created_at: string
  updated_at: string
}
