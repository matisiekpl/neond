import { Navigate, Outlet } from "react-router"
import { AuthenticatedLayout } from "~/components/AuthenticatedLayout"
import { useAuth } from "~/contexts/AuthContext"
import { Spinner } from "~/components/ui/spinner"

export default function DashboardLayout() {
  const { user, initialized } = useAuth()

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

  return (
    <AuthenticatedLayout>
      <Outlet />
    </AuthenticatedLayout>
  )
}
