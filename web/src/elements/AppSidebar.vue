<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
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
  Terminal,
} from 'lucide-vue-next'
import { useOrganizationStore } from '@/stores/organization.store'
import { useProjectStore } from '@/stores/project.store'
import CreateOrganizationDialog from '@/elements/CreateOrganizationDialog.vue'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
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
} from '@/components/ui/sidebar'

const route = useRoute()
const router = useRouter()
const organizationStore = useOrganizationStore()
const projectStore = useProjectStore()

const createOpen = ref(false)

const projectId = computed(() => route.params.projectId as string | undefined)
const currentProject = computed(() => projectStore.projects.find((p) => p.id === projectId.value))
const displayOrg = computed(() => organizationStore.organizations.find((o) => o.id === organizationStore.selectedOrganizationId))
</script>

<template>
  <Sidebar collapsible="icon" variant="inset">
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <SidebarMenuButton size="lg" class="data-[state=open]:bg-sidebar-accent cursor-pointer">
                <div class="flex aspect-square size-8 items-center justify-center rounded-none bg-sidebar-primary text-sidebar-primary-foreground">
                  <GalleryVerticalEnd class="size-4" />
                </div>
                <div class="grid flex-1 text-left text-xs leading-tight">
                  <span class="truncate font-medium">{{ displayOrg?.name ?? 'No organization' }}</span>
                  <span class="truncate text-muted-foreground">Organization</span>
                </div>
                <ChevronDown class="ml-auto size-4" />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent class="min-w-56 rounded-none">
              <DropdownMenuLabel>Organizations</DropdownMenuLabel>
              <DropdownMenuItem
                v-for="org in organizationStore.organizations"
                :key="org.id"
                @click="organizationStore.saveSelectedOrganization(org.id)"
              >
                <Building2 />
                <span class="flex-1">{{ org.name }}</span>
                <Check v-if="organizationStore.selectedOrganizationId === org.id" class="size-4" />
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem @click.prevent="createOpen = true">
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
              as-child
              :is-active="route.path === '/dashboard/projects'"
              tooltip="Projects"
            >
              <RouterLink to="/dashboard/projects">
                <LayoutDashboard />
                <span>Projects</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              as-child
              :is-active="route.path === '/dashboard/settings/organization'"
              tooltip="Organization"
            >
              <RouterLink to="/dashboard/settings/organization">
                <Settings />
                <span>Organization</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              as-child
              :is-active="route.path === '/dashboard/daemon'"
              tooltip="Daemon"
            >
              <RouterLink to="/dashboard/daemon">
                <Terminal />
                <span>Daemon</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>

      <SidebarGroup v-if="projectId">
        <SidebarGroupLabel>Project</SidebarGroupLabel>
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <SidebarMenuButton
                  class="border data-[state=open]:bg-sidebar-accent cursor-pointer"
                  :tooltip="currentProject?.name ?? 'Project'"
                >
                  <Database class="size-4" />
                  <span class="flex-1 truncate">{{ currentProject?.name ?? 'Unknown project' }}</span>
                  <ChevronDown class="ml-auto size-4" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent class="min-w-48 rounded-none" side="right" align="start">
                <DropdownMenuLabel>Projects</DropdownMenuLabel>
                <DropdownMenuItem
                  v-for="project in projectStore.projects"
                  :key="project.id"
                  @click="router.push(`/dashboard/projects/${project.id}`)"
                >
                  <span class="flex-1">{{ project.name }}</span>
                  <Check v-if="projectId === project.id" class="size-4" />
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem @click="router.push('/dashboard/projects')">
                  <LayoutDashboard />
                  All projects
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>

          <SidebarMenuItem class="mt-1">
            <SidebarMenuButton
              as-child
              :is-active="route.path === `/dashboard/projects/${projectId}`"
              tooltip="Details"
            >
              <RouterLink :to="`/dashboard/projects/${projectId}`">
                <Info />
                <span>Details</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>

          <SidebarMenuItem>
            <SidebarMenuButton
              as-child
              :is-active="route.path === `/dashboard/projects/${projectId}/settings`"
              tooltip="Settings"
            >
              <RouterLink :to="`/dashboard/projects/${projectId}/settings`">
                <Settings />
                <span>Settings</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>
    </SidebarContent>

    <SidebarFooter>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton as-child tooltip="Log out">
            <RouterLink to="/logout">
              <LogOut />
              <span>Log out</span>
            </RouterLink>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarFooter>
    <SidebarRail />
  </Sidebar>

  <CreateOrganizationDialog v-model:open="createOpen" />
</template>
