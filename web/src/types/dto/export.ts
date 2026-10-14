export type ExportFormat = 'plain' | 'custom'

export type ExportState =
  | 'idle'
  | 'preparing'
  | 'dumping'
  | 'finalizing'
  | 'completed'
  | 'failed'
  | 'cancelled'

export interface ExportRequest {
  database: string
  format: ExportFormat
  no_owner: boolean
  no_privileges: boolean
}

export interface ExportStatus {
  export_id: string | null
  state: ExportState
  format: ExportFormat | null
  database: string | null
  pg_version: string | null
  pg_dump_version: string | null
  started_at: string | null
  finished_at: string | null
  expires_at: string | null
  error: string | null
  progress: number
  current_table: string | null
  tables_total: number
  tables_done: number
  bytes_total_estimate: number
  bytes_written: number
  file_name: string | null
}

export interface ExportDownloadToken {
  token: string
  expires_at: string
}

export function isExportActive(state: ExportState | null | undefined): boolean {
  return state === 'preparing' || state === 'dumping' || state === 'finalizing'
}

export function pgMajorVersion(pgVersion: string | null | undefined): string | null {
  if (!pgVersion) return null
  return pgVersion.replace(/^v/i, '')
}
