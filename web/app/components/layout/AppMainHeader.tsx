import { useLocation } from "react-router"
import { SidebarTrigger } from "~/components/ui/sidebar"

const TITLES: Record<string, string> = {
  "/dashboard": "Dashboard",
  "/dashboard/projects": "Projects",
  "/dashboard/settings/organization": "Organization settings",
}

export function AppMainHeader() {
  const { pathname } = useLocation()
  const title = TITLES[pathname] ?? "Page"

  return (
    <header className="flex h-12 shrink-0 items-center gap-2 border-b px-2">
      <SidebarTrigger />
      <span className="text-xs font-medium">{title}</span>
    </header>
  )
}
