import * as React from "react"
import { useAuthStore } from "~/stores/auth-store"

export default function LogoutRoute() {
  const logout = useAuthStore((s) => s.logout)

  React.useEffect(() => {
    logout()
  }, [logout])

  return null
}
