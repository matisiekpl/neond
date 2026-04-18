export interface ExecuteSqlResponse {
  columns: string[]
  rows: (string | null)[][]
  rows_affected: number | null
  error: string | null
}
