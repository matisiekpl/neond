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
    void useOrganizationStore.getState().ensureOrganizations(user.name)
  }, [user?.id, user?.name])

  return <>{children}</>
}

export { useOrganization, useOrganizationStore } from "~/stores/organization-store"
