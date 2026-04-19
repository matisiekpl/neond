export interface RowUpdate {
  primaryKeyValues: Record<string, string | null>
  changedCells: Record<string, string | null>
}