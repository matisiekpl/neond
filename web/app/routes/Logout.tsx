import * as React from "react"
import { useAuth } from "~/contexts/AuthContext"

export default function LogoutRoute() {
  const { logout } = useAuth()

  React.useEffect(() => {
    logout()
  }, [logout])

  return null
}
