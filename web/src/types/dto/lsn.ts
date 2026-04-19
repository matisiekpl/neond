export type LsnKind = 'present' | 'future' | 'past' | 'nodata'

export interface LsnResponse {
  lsn: string
  kind: LsnKind
}