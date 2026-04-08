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
  organization_id: string
  organization_name: string
  project_id: string
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

export interface PendingShutdownInfo {
  wait_for_checkpoints: boolean
  requested_at: string
}

export interface DaemonState {
  hostname: string | null
  build_version: string
  storage: StorageInfo
  mappings: MappingInfo[]
  pending_shutdown: PendingShutdownInfo | null
  max_checkpoint_timeout: { secs: number; nanos: number } | null
}