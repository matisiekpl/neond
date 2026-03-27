import * as React from "react"
import { Link, useLocation, useNavigate, useParams } from "react-router"
import {
  Building2,
  Check,
  ChevronDown,
  Database,
  GalleryVerticalEnd,
  Info,
  LayoutDashboard,
  LogOut,
  Plus,
  Settings,
} from "lucide-react"
import { CreateOrganizationDialog } from "~/components/organization/CreateOrganizationDialog"
import { useOrganizationStore } from "~/stores/organization-store"
import { useProjectStore } from "~/stores/project-store"
import { useShallow } from "zustand/react/shallow"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "~/components/ui/sidebar"

export function AppSidebar() {
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()

  const { organizations, selectedOrganizationId, saveSelectedOrganization } =
    useOrganizationStore(
      useShallow((s) => ({
        organizations: s.organizations,
        selectedOrganizationId: s.selectedOrganizationId,
        saveSelectedOrganization: s.saveSelectedOrganization,
      })),
    )
  const projects = useProjectStore((s) => s.projects)

  const [createOpen, setCreateOpen] = React.useState(false)

  const displayOrg = organizations.find((o) => o.id === selectedOrganizationId)
  const title = displayOrg?.name ?? "No organization"
  const subtitle = "Organization"

  const currentProject = projects.find((p) => p.id === projectId)

  return (
    <>
      <Sidebar collapsible="icon" variant="inset">
        <SidebarHeader>
          <SidebarMenu>
            <SidebarMenuItem>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <SidebarMenuButton
                    size="lg"
                    className="data-[state=open]:bg-sidebar-accent"
                  >
                    <div className="flex aspect-square size-8 items-center justify-center rounded-none bg-sidebar-primary text-sidebar-primary-foreground">
                      <GalleryVerticalEnd className="size-4" />
                    </div>
                    <div className="grid flex-1 text-left text-xs leading-tight">
                      <span className="truncate font-medium">{title}</span>
                      <span className="truncate text-muted-foreground">
                        {subtitle}
                      </span>
                    </div>
                    <ChevronDown className="ml-auto size-4" />
                  </SidebarMenuButton>
                </DropdownMenuTrigger>
                <DropdownMenuContent className="min-w-56 rounded-none">
                  <DropdownMenuLabel>Organizations</DropdownMenuLabel>
                  {organizations.map((org) => (
                    <DropdownMenuItem
                      key={org.id}
                      onClick={() => saveSelectedOrganization(org.id)}
                    >
                      <Building2 />
                      <span className="flex-1">{org.name}</span>
                      {selectedOrganizationId === org.id ? (
                        <Check className="size-4" />
                      ) : null}
                    </DropdownMenuItem>
                  ))}
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onSelect={(e) => {
                      e.preventDefault()
                      setCreateOpen(true)
                    }}
                  >
                    <Plus />
                    Create organization
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarHeader>
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupLabel>Platform</SidebarGroupLabel>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={pathname === "/dashboard/projects"}
                  tooltip="Projects"
                >
                  <Link to="/dashboard/projects">
                    <LayoutDashboard />
                    <span>Projects</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={
                    pathname === "/dashboard/settings/organization"
                  }
                  tooltip="Organization"
                >
                  <Link to="/dashboard/settings/organization">
                    <Settings />
                    <span>Organization</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroup>
          {projectId ? (
            <SidebarGroup>
              <SidebarGroupLabel>Project</SidebarGroupLabel>
              <SidebarMenu>
                <SidebarMenuItem>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <SidebarMenuButton
                        className="border data-[state=open]:bg-sidebar-accent"
                        tooltip={currentProject?.name ?? "Project"}
                      >
                        <Database className="size-4" />
                        <span className="flex-1 truncate">
                          {currentProject?.name ?? "Unknown project"}
                        </span>
                        <ChevronDown className="ml-auto size-4" />
                      </SidebarMenuButton>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent className="min-w-48 rounded-none" side="right" align="start">
                      <DropdownMenuLabel>Projects</DropdownMenuLabel>
                      {projects.map((project) => (
                        <DropdownMenuItem
                          key={project.id}
                          onClick={() => navigate(`/dashboard/projects/${project.id}`)}
                        >
                          <span className="flex-1">{project.name}</span>
                          {projectId === project.id ? (
                            <Check className="size-4" />
                          ) : null}
                        </DropdownMenuItem>
                      ))}
                      <DropdownMenuSeparator />
                      <DropdownMenuItem onClick={() => navigate("/dashboard/projects")}>
                        <LayoutDashboard />
                        All projects
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </SidebarMenuItem>
                <SidebarMenuItem className="mt-1">
                  <SidebarMenuButton
                    asChild
                    isActive={pathname === `/dashboard/projects/${projectId}`}
                    tooltip="Details"
                  >
                    <Link to={`/dashboard/projects/${projectId}`}>
                      <Info />
                      <span>Details</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem>
                  <SidebarMenuButton
                    asChild
                    isActive={pathname === `/dashboard/projects/${projectId}/settings`}
                    tooltip="Settings"
                  >
                    <Link to={`/dashboard/projects/${projectId}/settings`}>
                      <Settings />
                      <span>Settings</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>
          ) : null}
        </SidebarContent>
        <SidebarFooter>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton asChild tooltip="Log out">
                <Link to="/logout">
                  <LogOut />
                  <span>Log out</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarFooter>
        <SidebarRail />
      </Sidebar>
      <CreateOrganizationDialog open={createOpen} onOpenChange={setCreateOpen} />
    </>
  )
}
