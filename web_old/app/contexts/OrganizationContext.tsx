import * as React from "react"
import { useAuthStore } from "~/stores/auth-store"
import { useOrganizationStore } from "~/stores/organization-store"

export function OrganizationProvider({ children }: { children: React.ReactNode }) {
  const user = useAuthStore((s) => s.user)

  React.useEffect(() => {
    if (!user) {
      useOrganizationStore.getState().reset()
      return
    }
    useOrganizationStore.setState({ loaded: false })
    void useOrganizationStore
      .getState()
      .loadOrganizations()
      .finally(() => {
        useOrganizationStore.setState({ loaded: true })
      })
  }, [user?.id])

  return <>{children}</>
}

