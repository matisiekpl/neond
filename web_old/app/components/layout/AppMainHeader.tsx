import { useLocation, useParams } from "react-router"
import { SidebarTrigger } from "~/components/ui/sidebar"
import { useProjectStore } from "~/stores/project-store"

const TITLES: Record<string, string> = {
  "/dashboard": "Dashboard",
  "/dashboard/projects": "Projects",
  "/dashboard/settings/organization": "Organization settings",
}

export function AppMainHeader() {
  const { pathname } = useLocation()
  const { projectId } = useParams<{ projectId: string }>()
  const projects = useProjectStore((s) => s.projects)

  let title = TITLES[pathname] ?? "Page"
  if (projectId) {
    const project = projects.find((p) => p.id === projectId)
    const projectName = project?.name ?? "Project"
    title = pathname.endsWith("/settings") ? `${projectName} — Settings` : projectName
  }

  return (
    <header className="flex h-12 shrink-0 items-center gap-2 border-b px-2">
      <SidebarTrigger />
      <span className="text-xs font-medium">{title}</span>
    </header>
  )
}
