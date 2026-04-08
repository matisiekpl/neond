import type { BranchStatus } from '@/types/models/branch'

export interface LocalStorageInfo {
  type: 'local'
  used_bytes: number
  free_bytes: number
  used_percent: number
}

export interface RemoteStorageInfo {
  type: 'remote'
  bucket: string
  region: string
  aws_access_key_id: string
}

export type StorageInfo = LocalStorageInfo | RemoteStorageInfo

export interface MappingInfo {
  branch_id: string
  organization_name: string
  project_name: string
  branch_name: string
  slug: string
  endpoint_status: BranchStatus
  port: number | null
  sni: string | null
  last_record_lsn: string
  remote_consistent_lsn_visible: string
  current_logical_size: number
}

export interface DaemonState {
  storage: StorageInfo
  mappings: MappingInfo[]
}