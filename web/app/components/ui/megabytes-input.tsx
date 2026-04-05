import * as React from "react"
import { Input } from "~/components/ui/input"

const MB = 1024 * 1024

function bytesToMb(bytes: string): string {
  if (!bytes.trim()) return ""
  const n = Number(bytes)
  if (isNaN(n)) return ""
  return String(n / MB)
}

function mbToBytes(mb: string): string {
  if (!mb.trim()) return ""
  const n = Number(mb)
  if (isNaN(n)) return ""
  return String(Math.round(n * MB))
}

type MegabytesInputProps = Omit<React.ComponentProps<typeof Input>, "value" | "onChange"> & {
  value: string
  onChange: (v: string) => void
}

export function MegabytesInput({ value, onChange, ...props }: MegabytesInputProps) {
  return (
    <div className="flex items-center gap-2">
      <Input
        type="number"
        min={0}
        value={bytesToMb(value)}
        onChange={(e) => onChange(mbToBytes(e.target.value))}
        {...props}
      />
      <span className="shrink-0 text-sm text-muted-foreground">MB</span>
    </div>
  )
}