import * as React from "react"
import { Link, useLocation } from "react-router"
import {
  Building2,
  Check,
  ChevronDown,
  GalleryVerticalEnd,
  LayoutDashboard,
  LogOut,
  Plus,
  Settings,
} from "lucide-react"
import { CreateOrganizationDialog } from "~/components/organization/CreateOrganizationDialog"
import { useOrganizationStore } from "~/stores/organization-store"
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
  const { organizations, selectedOrganizationId, saveSelectedOrganization } =
    useOrganizationStore(
      useShallow((s) => ({
        organizations: s.organizations,
        selectedOrganizationId: s.selectedOrganizationId,
        saveSelectedOrganization: s.saveSelectedOrganization,
      })),
    )
  const [createOpen, setCreateOpen] = React.useState(false)

  const displayOrg = organizations.find((o) => o.id === selectedOrganizationId)
  const title = displayOrg?.name ?? "No organization"
  const subtitle = "Organization"

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
                  isActive={pathname === "/dashboard"}
                  tooltip="Dashboard"
                >
                  <Link to="/dashboard">
                    <LayoutDashboard />
                    <span>Dashboard</span>
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
