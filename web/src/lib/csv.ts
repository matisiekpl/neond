export function toCsv(columns: string[], rows: (string | null)[][]): string {
  const escape = (value: string | null) => {
    if (value === null) return ''
    const needsQuote = /[",\n\r]/.test(value)
    const escaped = value.replace(/"/g, '""')
    return needsQuote ? `"${escaped}"` : escaped
  }
  const header = columns.map(escape).join(',')
  const body = rows.map((row) => row.map(escape).join(',')).join('\n')
  return `${header}\n${body}`
}
