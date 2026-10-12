export interface CompactionStatus {
  running: boolean
  started_at: string | null
  finished_at: string | null
  error: string | null
  size_before: number | null
  size_after: number | null
}
