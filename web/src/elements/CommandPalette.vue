<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useDark, useToggle } from '@vueuse/core'
import {
  Activity,
  ArrowRight,
  ChevronLeft,
  Clipboard,
  Database,
  LayoutDashboard,
  LogOut,
  Moon,
  Plus,
  ScrollText,
  Settings,
  Sun,
  Terminal,
  Users,
  Zap,
} from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from '@/components/ui/command'
import { useCommandPalette } from '@/composables/useCommandPalette'
import { useAuthStore } from '@/stores/auth.store'
import { useProjectStore } from '@/stores/project.store'
import { useBranchStore } from '@/stores/branch.store'
import CreateProjectDialog from '@/elements/CreateProjectDialog.vue'
import CreateOrganizationDialog from '@/elements/CreateOrganizationDialog.vue'
import ShutdownDaemonDialog from '@/elements/ShutdownDaemonDialog.vue'

const { open } = useCommandPalette()
const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const projectStore = useProjectStore()
const branchStore = useBranchStore()
const isDark = useDark()
const toggleDark = useToggle(isDark)

const currentPage = ref('')

const createProjectOpen = ref(false)
const createOrganizationOpen = ref(false)
const shutdownDaemonOpen = ref(false)

const organizationId = computed(() => route.params.organizationId as string)
const projectId = computed(() => route.params.projectId as string | undefined)

const daemonLogComponents = [
  'storage_broker',
  'storage_controller',
  'pageserver',
  'safekeeper',
  'storage_controller_db',
  'management_db',
]

const inputPlaceholder = computed(() =>
  currentPage.value === 'daemonLogs' ? 'Select component…' : 'Type a command or search…',
)

function close() {
  open.value = false
  currentPage.value = ''
}

function navigate(name: string, params?: Record<string, string>) {
  router.push({ name, params: { organizationId: organizationId.value, ...params } })
  close()
}

function copyConnectionString(branchId: string) {
  const branch = branchStore.branches.find((b) => b.id === branchId)
  if (!branch?.connection_string) return
  navigator.clipboard.writeText(branch.connection_string)
  toast.success('Connection string copied')
  close()
}

function onInputKeydown(event: KeyboardEvent) {
  const isEmpty = (event.target as HTMLInputElement).value === ''
  if ((event.key === 'Backspace' || event.key === 'ArrowLeft') && isEmpty) {
    currentPage.value = ''
  }
}
</script>

<template>
  <CommandDialog :open="open" @update:open="(val) => { open = val; if (!val) currentPage = '' }">
    <CommandInput :placeholder="inputPlaceholder" @keydown="onInputKeydown" />
    <CommandList>
      <CommandEmpty>No results found.</CommandEmpty>

      <template v-if="currentPage === ''">
        <CommandGroup heading="Navigate">
          <CommandItem value="go to projects" @select="navigate('projects.list')">
            <LayoutDashboard />
            Projects
          </CommandItem>
          <CommandItem value="go to organization settings" @select="navigate('settings.organization')">
            <Settings />
            Organization Settings
          </CommandItem>
          <CommandItem value="go to daemon" @select="navigate('daemon')">
            <Terminal />
            Daemon
          </CommandItem>
          <CommandItem value="go to daemon monitoring" @select="navigate('daemon.monitoring')">
            <Activity />
            Daemon — Monitoring
          </CommandItem>
          <CommandItem value="daemon logs" @select="currentPage = 'daemonLogs'">
            <ScrollText />
            Daemon Logs
            <CommandShortcut>
              <ArrowRight class="size-3.5" />
            </CommandShortcut>
          </CommandItem>
          <CommandItem v-if="authStore.user?.is_admin" value="go to users" @select="navigate('users')">
            <Users />
            Users
          </CommandItem>
        </CommandGroup>

        <CommandGroup v-if="projectStore.projects.length > 0" heading="Projects">
          <CommandItem
            v-for="project in projectStore.projects"
            :key="project.id"
            :value="`go to project ${project.name}`"
            @select="navigate('projects.show', { projectId: project.id })"
          >
            <Database />
            {{ project.name }}
          </CommandItem>
        </CommandGroup>

        <CommandGroup v-if="projectId && branchStore.branches.length > 0" heading="Branches">
          <template v-for="branch in branchStore.branches" :key="branch.id">
            <CommandItem
              :value="`go to branch ${branch.name} data`"
              @select="navigate('projects.branches.data', { projectId: projectId!, branchId: branch.id })"
            >
              <Database />
              {{ branch.name }} — Data
            </CommandItem>
            <CommandItem
              :value="`go to branch ${branch.name} sql`"
              @select="navigate('projects.branches.sql', { projectId: projectId!, branchId: branch.id })"
            >
              <Terminal />
              {{ branch.name }} — SQL
            </CommandItem>
            <CommandItem
              :value="`go to branch ${branch.name} metrics`"
              @select="navigate('projects.branches.metrics', { projectId: projectId!, branchId: branch.id })"
            >
              <Activity />
              {{ branch.name }} — Metrics
            </CommandItem>
            <CommandItem
              :value="`go to branch ${branch.name} logs`"
              @select="navigate('projects.branches.logs', { projectId: projectId!, branchId: branch.id })"
            >
              <ScrollText />
              {{ branch.name }} — Logs
            </CommandItem>
            <CommandItem
              v-if="branch.connection_string"
              :value="`copy connection string ${branch.name}`"
              @select="copyConnectionString(branch.id)"
            >
              <Clipboard />
              {{ branch.name }} — Copy Connection String
            </CommandItem>
          </template>
        </CommandGroup>

        <CommandGroup heading="Create">
          <CommandItem value="create project" @select="() => { close(); createProjectOpen = true }">
            <Plus />
            Create Project
          </CommandItem>
          <CommandItem value="create organization" @select="() => { close(); createOrganizationOpen = true }">
            <Plus />
            Create Organization
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading="Actions">
          <CommandItem value="toggle dark mode" @select="() => { toggleDark(); close() }">
            <Sun v-if="isDark" />
            <Moon v-else />
            Toggle Dark Mode
          </CommandItem>
          <CommandItem value="log out" @select="() => { router.push({ name: 'logout' }); close() }">
            <LogOut />
            Log Out
          </CommandItem>
          <CommandItem value="shutdown daemon" @select="() => { close(); shutdownDaemonOpen = true }">
            <Zap />
            Shutdown Daemon
          </CommandItem>
        </CommandGroup>
      </template>

      <template v-if="currentPage === 'daemonLogs'">
        <CommandGroup heading="Daemon Logs — Select Component">
          <CommandItem value="back" @select="currentPage = ''">
            <ChevronLeft />
            Back
          </CommandItem>
        </CommandGroup>
        <CommandGroup heading="Component">
          <CommandItem
            v-for="component in daemonLogComponents"
            :key="component"
            :value="`daemon logs ${component}`"
            @select="navigate('daemon.logs', { component })"
          >
            <ScrollText />
            {{ component }}
          </CommandItem>
        </CommandGroup>
      </template>
    </CommandList>
  </CommandDialog>

  <CreateProjectDialog v-model:open="createProjectOpen" />
  <CreateOrganizationDialog v-model:open="createOrganizationOpen" />
  <ShutdownDaemonDialog v-model:open="shutdownDaemonOpen" :awaiting-count="0" />
</template>
