import * as React from "react"
import { useNavigate } from "react-router"
import { useAuthStore } from "~/stores/auth-store"

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const navigate = useNavigate()
  const setNavigate = useAuthStore((s) => s.setNavigate)
  const bootstrap = useAuthStore((s) => s.bootstrap)

  React.useEffect(() => {
    setNavigate(navigate)
    return () => setNavigate(null)
  }, [navigate, setNavigate])

  React.useEffect(() => {
    void bootstrap()
  }, [bootstrap])

  return <>{children}</>
}

export { useAuth, useAuthStore } from "~/stores/auth-store"
