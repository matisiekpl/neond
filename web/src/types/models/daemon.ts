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
  organization_name: string
  project_name: string
  branch_name: string
  port: number
  sni: string | null
}

export interface DaemonState {
  storage: StorageInfo
  mappings: MappingInfo[]
}