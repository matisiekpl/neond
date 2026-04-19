<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  Building2,
  Check,
  ChevronDown,
  Database,
  GalleryVerticalEnd,
  GitBranch,
  Github,
  LayoutDashboard,
  LogOut,
  Plus,
  Settings,
  Terminal,
  Users,
} from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { useProjectStore } from '@/stores/project.store'
import { useBranchStore } from '@/stores/branch.store'
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
const authStore = useAuthStore()
const organizationStore = useOrganizationStore()
const projectStore = useProjectStore()
const branchStore = useBranchStore()

const createOpen = ref(false)

const organizationId = computed(() => route.params.organizationId as string)
const projectId = computed(() => route.params.projectId as string | undefined)
const branchId = computed(() => route.params.branchId as string | undefined)
const currentProject = computed(() => projectStore.projects.find((p) => p.id === projectId.value))
const currentBranch = computed(() => branchStore.branches.find((b) => b.id === branchId.value))
const displayOrg = computed(() => organizationStore.organizations.find((o) => o.id === organizationId.value))
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
                @click="router.push({ name: 'projects.list', params: { organizationId: org.id } })"
              >
                <Building2 />
                <span class="flex-1">{{ org.name }}</span>
                <Check v-if="organizationId === org.id" class="size-4" />
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
              :is-active="route.name === 'projects.list'"
              tooltip="Projects"
            >
              <RouterLink :to="{ name: 'projects.list', params: { organizationId } }">
                <LayoutDashboard />
                <span>Projects</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              as-child
              :is-active="route.name === 'settings.organization'"
              tooltip="Organization"
            >
              <RouterLink :to="{ name: 'settings.organization', params: { organizationId } }">
                <Settings />
                <span>Organization</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              as-child
              :is-active="route.name === 'daemon'"
              tooltip="Daemon"
            >
              <RouterLink :to="{ name: 'daemon', params: { organizationId } }">
                <Terminal />
                <span>Daemon</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem v-if="authStore.user?.is_admin">
            <SidebarMenuButton
              as-child
              :is-active="route.name === 'users'"
              tooltip="Users"
            >
              <RouterLink :to="{ name: 'users', params: { organizationId } }">
                <Users />
                <span>Users</span>
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
                  @click="router.push({ name: 'projects.show', params: { organizationId, projectId: project.id } })"
                >
                  <span class="flex-1">{{ project.name }}</span>
                  <Check v-if="projectId === project.id" class="size-4" />
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem @click="router.push({ name: 'projects.list', params: { organizationId } })">
                  <LayoutDashboard />
                  All projects
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>

          <SidebarMenuItem class="mt-1">
            <SidebarMenuButton
              as-child
              :is-active="route.name === 'projects.show'"
              tooltip="Branches"
            >
              <RouterLink :to="{ name: 'projects.show', params: { organizationId, projectId } }">
                <GitBranch />
                <span>Branches</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>

          <SidebarMenuItem>
            <SidebarMenuButton
              as-child
              :is-active="route.name === 'projects.settings'"
              tooltip="Settings"
            >
              <RouterLink :to="{ name: 'projects.settings', params: { organizationId, projectId } }">
                <Settings />
                <span>Settings</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>

      <SidebarGroup v-if="branchId">
        <SidebarGroupLabel>Branch</SidebarGroupLabel>
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <SidebarMenuButton
                  class="border data-[state=open]:bg-sidebar-accent cursor-pointer"
                  :tooltip="currentBranch?.name ?? 'Branch'"
                >
                  <GitBranch class="size-4" />
                  <span class="flex-1 truncate">{{ currentBranch?.name ?? 'Unknown branch' }}</span>
                  <ChevronDown class="ml-auto size-4" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent class="min-w-48 rounded-none" side="right" align="start">
                <DropdownMenuLabel>Branches</DropdownMenuLabel>
                <DropdownMenuItem
                  v-for="branch in branchStore.branches"
                  :key="branch.id"
                  @click="router.push({ name: 'projects.branches.data', params: { organizationId, projectId, branchId: branch.id } })"
                >
                  <span class="flex-1">{{ branch.name }}</span>
                  <Check v-if="branchId === branch.id" class="size-4" />
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem @click="router.push({ name: 'projects.show', params: { organizationId, projectId } })">
                  <LayoutDashboard />
                  All branches
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>

          <SidebarMenuItem class="mt-1">
            <SidebarMenuButton
              as-child
              :is-active="route.name === 'projects.branches.data'"
              tooltip="Data"
            >
              <RouterLink :to="{ name: 'projects.branches.data', params: { organizationId, projectId, branchId } }">
                <Database />
                <span>Data</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>

          <SidebarMenuItem>
            <SidebarMenuButton
              as-child
              :is-active="route.name === 'projects.branches.sql'"
              tooltip="SQL"
            >
              <RouterLink :to="{ name: 'projects.branches.sql', params: { organizationId, projectId, branchId } }">
                <Terminal />
                <span>SQL</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>
    </SidebarContent>

    <SidebarFooter>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton as-child tooltip="GitHub">
            <a href="https://github.com/matisiekpl/neond" target="_blank" rel="noopener noreferrer">
              <Github />
              <span>GitHub</span>
            </a>
          </SidebarMenuButton>
        </SidebarMenuItem>
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