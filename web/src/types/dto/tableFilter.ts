export type FilterOperator =
  | 'equals'
  | 'not_equals'
  | 'contains'
  | 'starts_with'
  | 'greater_than'
  | 'less_than'
  | 'is_null'
  | 'is_not_null'

export interface TableFilter {
  column: string
  operator: FilterOperator
  value: string
}

export type SortDirection = 'asc' | 'desc'

export interface TableSort {
  column: string
  direction: SortDirection
}
