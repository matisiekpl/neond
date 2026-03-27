import { Navigate, Outlet, useLocation } from "react-router"
import { useShallow } from "zustand/react/shallow"
import { AuthenticatedLayout } from "~/components/layout/AuthenticatedLayout"
import { useAuthStore } from "~/stores/auth-store"
import { useOrganizationStore } from "~/stores/organization-store"
import { Spinner } from "~/components/ui/spinner"

const SETUP_ORG_PATH = "/dashboard/setup-organization"

export default function DashboardLayout() {
  const { user, initialized } = useAuthStore(
    useShallow((s) => ({ user: s.user, initialized: s.initialized })),
  )
  const { organizations, loaded } = useOrganizationStore(
    useShallow((s) => ({ organizations: s.organizations, loaded: s.loaded })),
  )
  const location = useLocation()
  const onSetupRoute = location.pathname === SETUP_ORG_PATH

  if (!initialized) {
    return (
      <div className="flex min-h-svh items-center justify-center">
        <Spinner className="size-8" />
      </div>
    )
  }

  if (!user) {
    return <Navigate to="/login" replace />
  }

  if (!loaded) {
    return (
      <div className="flex min-h-svh items-center justify-center">
        <Spinner className="size-8" />
      </div>
    )
  }

  if (!onSetupRoute && organizations.length === 0) {
    return <Navigate to={SETUP_ORG_PATH} replace />
  }

  if (onSetupRoute && organizations.length > 0) {
    return <Navigate to="/dashboard" replace />
  }

  if (onSetupRoute) {
    return <Outlet />
  }

  return (
    <AuthenticatedLayout>
      <Outlet />
    </AuthenticatedLayout>
  )
}
