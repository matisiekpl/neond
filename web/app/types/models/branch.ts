export interface Branch {
  id: string
  project_id: string
  name: string
  slug: string
  parent_branch_id: string | null
  remote_consistent_lsn_visible: string
  last_record_lsn: string
  created_at: string
  updated_at: string
}
