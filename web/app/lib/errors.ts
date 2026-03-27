import type { AxiosError } from "axios"

export function getAppError(err: unknown): string {
  const ax = err as AxiosError<{ message?: string }>
  const data = ax.response?.data
  if (data && typeof data === "object" && "message" in data && data.message) {
    const m = String(data.message)
    return m.charAt(0).toUpperCase() + m.slice(1)
  }
  if (err instanceof Error) {
    const m = err.message
    return m.charAt(0).toUpperCase() + m.slice(1)
  }
  return "Something went wrong"
}
