<script setup lang="ts">
import {ref, computed, watch, onUnmounted} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import {useTitle} from '@vueuse/core'
import {toast} from 'vue-sonner'
import {
  BookDown, Cloud, Copy, GitBranchPlus, Loader2, MoreVertical, Pencil, Play, Square, Trash2,
} from 'lucide-vue-next'
import {useProjectStore} from '@/stores/project.store'
import {useOrganizationStore} from '@/stores/organization.store'
import {useBranchStore} from '@/stores/branch.store'
import {getAppError} from '@/api/utils'
import type {Branch, BranchStatus} from '@/types/models/branch'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {Button} from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from '@/components/ui/hover-card'
import {Input} from '@/components/ui/input'
import {Label} from '@/components/ui/label'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const organizationStore = useOrganizationStore()
const branchStore = useBranchStore()

const projectId = computed(() => route.params.projectId as string)

const project = computed(() => projectStore.projects.find((p) => p.id === projectId.value))

useTitle(computed(() => project.value ? `${project.value.name} — neond` : 'neond'))

const STATUS_CONFIG: Record<BranchStatus, { label: string; className: string }> = {
  running: {label: 'Running', className: 'bg-green-500'},
  starting: {label: 'Starting', className: 'bg-amber-400'},
  stopping: {label: 'Stopping', className: 'bg-amber-400'},
  stopped: {label: 'Stopped', className: 'bg-muted-foreground'},
  failed: {label: 'Failed', className: 'bg-destructive'},
}

type BranchNode = Branch & { children: BranchNode[] }

function buildTree(branches: Branch[]): BranchNode[] {
  const map = new Map<string, BranchNode>()
  for (const b of branches) map.set(b.id, {...b, children: []})
  const roots: BranchNode[] = []
  for (const node of map.values()) {
    if (node.parent_branch_id && map.has(node.parent_branch_id)) {
      map.get(node.parent_branch_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  }
  return roots
}

function flattenTree(nodes: BranchNode[], depth = 0): { branch: BranchNode; depth: number }[] {
  const result: { branch: BranchNode; depth: number }[] = []
  for (const node of nodes) {
    result.push({branch: node, depth})
    result.push(...flattenTree(node.children, depth + 1))
  }
  return result
}

const rows = computed(() => flattenTree(buildTree(branchStore.branches)))

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${(bytes / 1024 ** i).toFixed(i === 0 ? 0 : 1)} ${units[i]}`
}

function formatDate(d: string) {
  return new Date(d).toLocaleDateString(undefined, {year: 'numeric', month: 'short', day: 'numeric'})
}

const createOpen = ref(false)
const createName = ref('')
const createSubmitting = ref(false)

const deleteOpen = ref(false)
const deleteId = ref<string | null>(null)
const deleting = ref(false)

const branchFromOpen = ref(false)
const branchFromParent = ref<Branch | null>(null)
const branchFromName = ref('')
const branchFromSubmitting = ref(false)

const renameOpen = ref(false)
const renameBranch = ref<Branch | null>(null)
const renameName = ref('')
const renameSubmitting = ref(false)

watch(createOpen, (val) => {
  if (!val) createName.value = ''
})
watch(branchFromOpen, (val) => {
  if (!val) branchFromName.value = ''
})
watch(renameOpen, (val) => {
  if (!val) renameName.value = ''
})

function openCreate() {
  createName.value = branchStore.branches.length === 0 ? 'production' : ''
  createOpen.value = true
}

function openDelete(branchId: string) {
  deleteId.value = branchId
  deleteOpen.value = true
}

function openBranchFrom(branch: Branch) {
  branchFromParent.value = branch
  branchFromName.value = ''
  branchFromOpen.value = true
}

function openRename(branch: Branch) {
  renameBranch.value = branch
  renameName.value = branch.name
  renameOpen.value = true
}

async function submitCreate() {
  if (!organizationStore.selectedOrganizationId || !projectId.value) return
  const trimmed = createName.value.trim()
  if (!trimmed) return
  createSubmitting.value = true
  try {
    await branchStore.createBranch(organizationStore.selectedOrganizationId, projectId.value, trimmed)
    createOpen.value = false
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    createSubmitting.value = false
  }
}

async function submitBranchFrom() {
  if (!organizationStore.selectedOrganizationId || !projectId.value || !branchFromParent.value) return
  const trimmed = branchFromName.value.trim()
  if (!trimmed) return
  branchFromSubmitting.value = true
  try {
    await branchStore.createBranch(organizationStore.selectedOrganizationId, projectId.value, trimmed, branchFromParent.value.id)
    branchFromOpen.value = false
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    branchFromSubmitting.value = false
  }
}

async function submitRename() {
  if (!organizationStore.selectedOrganizationId || !projectId.value || !renameBranch.value) return
  const trimmed = renameName.value.trim()
  if (!trimmed || trimmed === renameBranch.value.name) return
  renameSubmitting.value = true
  try {
    await branchStore.renameBranch(organizationStore.selectedOrganizationId, projectId.value, renameBranch.value.id, trimmed)
    renameOpen.value = false
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    renameSubmitting.value = false
  }
}

async function confirmDelete() {
  if (!organizationStore.selectedOrganizationId || !projectId.value || !deleteId.value) return
  deleting.value = true
  try {
    await branchStore.deleteBranch(organizationStore.selectedOrganizationId, projectId.value, deleteId.value)
    deleteOpen.value = false
  } finally {
    deleting.value = false
  }
}

let pollInterval: ReturnType<typeof setInterval> | null = null

watch(
  [() => organizationStore.selectedOrganizationId, projectId],
  ([orgId, pid]) => {
    if (!orgId || !pid) return
    branchStore.fetchBranches(orgId, pid)
    if (pollInterval) clearInterval(pollInterval)
    pollInterval = setInterval(() => {
      branchStore.fetchBranches(orgId, pid, true)
    }, 500)
  },
  {immediate: true},
)

watch(() => organizationStore.selectedOrganizationId, (orgId) => {
  if (orgId) projectStore.fetchProjects(orgId)
}, {immediate: true})

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
})

function copyConnectionString(branch: Branch) {
  if (branch.connection_string) {
    navigator.clipboard.writeText(branch.connection_string);
    toast.success('Connection string copied')
  }
}
</script>

<template>
  <div v-if="projectStore.loading" class="flex justify-center py-12">
    <Loader2 class="size-6 animate-spin"/>
  </div>

  <div
    v-else-if="!project"
    class="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center"
  >
    <p class="text-sm font-medium">Project not found</p>
    <p class="mt-1 text-sm text-muted-foreground">
      This project may have been deleted or you don't have access to it.
    </p>
    <button
      type="button"
      class="mt-4 text-sm underline underline-offset-4"
      @click="router.push('/dashboard/projects')"
    >
      Back to projects
    </button>
  </div>

  <div v-else class="space-y-6">
    <div>
      <h1 class="text-lg font-semibold">{{ project.name }}</h1>
      <div class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted-foreground">
        <span>PostgreSQL {{ project.pg_version.replace(/^V/i, '') }}</span>
        <span>·</span>
        <span>Created {{ new Date(project.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }) }}</span>
        <span>·</span>
        <span class="font-mono text-xs">{{ project.id }}</span>
      </div>
    </div>

    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold">Branches</h2>
        <Button type="button" size="sm" class="cursor-pointer" :disabled="createSubmitting" @click="openCreate">
          <Loader2 v-if="createSubmitting" class="mr-1.5 size-3.5 animate-spin"/>
          New branch
        </Button>
      </div>

      <div v-if="branchStore.loading && branchStore.branches.length === 0" class="flex justify-center py-8">
        <Loader2 class="size-5 animate-spin"/>
      </div>

      <div
        v-else-if="branchStore.branches.length === 0"
        class="flex min-h-24 w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-8 text-center"
      >
        <p class="text-sm text-muted-foreground">No branches yet.</p>
      </div>

      <div
        v-else
        :class="`relative border transition-opacity ${branchStore.loading ? 'opacity-50' : ''}`"
      >
        <div v-if="branchStore.loading" class="absolute inset-0 z-10 flex items-center justify-center">
          <Loader2 class="size-5 animate-spin"/>
        </div>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead class="w-28">Status</TableHead>
              <TableHead class="w-24">Size</TableHead>
              <TableHead class="w-44">Durability</TableHead>
              <TableHead class="w-28">Created</TableHead>
              <TableHead class="w-10"/>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="{ branch, depth } in rows" :key="branch.id">
              <TableCell class="font-medium">
                <span class="flex items-center gap-1.5" :style="{ paddingLeft: `${depth * 20}px` }">
                  <span v-if="depth > 0" class="shrink-0 text-muted-foreground">╰</span>
                  <span>{{ branch.name }}</span>
                </span>
              </TableCell>
              <TableCell class="w-28">
                <span class="flex items-center gap-1.5">
                  <span
                    :class="`inline-block size-2 shrink-0 rounded-full ${STATUS_CONFIG[branch.endpoint_status].className}`"/>
                  <span class="text-xs text-muted-foreground">{{ STATUS_CONFIG[branch.endpoint_status].label }}</span>
                </span>
              </TableCell>
              <TableCell class="w-24 text-xs text-muted-foreground">
                {{ formatBytes(branch.current_logical_size) }}
              </TableCell>
              <TableCell class="w-44">
                <HoverCard :open-delay="0" :close-delay="500">
                  <HoverCardTrigger as-child>
                    <span
                      :class="`inline-flex cursor-default items-center rounded border px-1.5 py-0.5 text-xs font-medium ${
                        branch.remote_consistent_lsn_visible === branch.last_record_lsn
                          ? 'border-green-500/30 bg-green-500/10 text-green-600'
                          : 'border-amber-500/30 bg-amber-500/10 text-amber-600'
                      }`"
                    >
                      {{
                        branch.remote_consistent_lsn_visible === branch.last_record_lsn ? 'checkpointed' : 'awaiting checkpoint'
                      }}
                    </span>
                  </HoverCardTrigger>
                  <HoverCardContent class="w-72">
                    <div class="flex flex-col gap-4">
                      <div class="flex w-full items-center gap-3">
                        <div class="flex w-20 flex-col items-center gap-1">
                          <span class="text-[10px] font-medium text-muted-foreground">WAL</span>
                          <BookDown class="size-5 shrink-0 text-green-500"/>
                          <span class="w-full truncate text-center font-mono text-[10px] text-muted-foreground">{{
                              branch.last_record_lsn
                            }}</span>
                        </div>
                        <div class="flex-1 self-start pt-[22px]">
                          <div
                            class="h-px w-full"
                            :style="{
                              backgroundImage: `radial-gradient(circle, ${branch.remote_consistent_lsn_visible === branch.last_record_lsn ? 'rgb(34 197 94)' : 'rgb(156 163 175)'} 1px, transparent 1px)`,
                              backgroundSize: '6px 1px',
                              animation: 'dash 1s linear infinite',
                            }"
                          />
                        </div>
                        <div class="flex w-20 flex-col items-center gap-1">
                          <span class="text-[10px] font-medium text-muted-foreground">Storage</span>
                          <Cloud
                            :class="`size-5 shrink-0 ${branch.remote_consistent_lsn_visible === branch.last_record_lsn ? 'text-green-500' : 'text-muted-foreground'}`"/>
                          <span class="w-full truncate text-center font-mono text-[10px] text-muted-foreground">{{
                              branch.remote_consistent_lsn_visible
                            }}</span>
                        </div>
                      </div>
                      <p class="text-center text-xs text-muted-foreground">
                        <template v-if="branch.remote_consistent_lsn_visible === branch.last_record_lsn">
                          All WAL records have been durably stored.
                        </template>
                        <template v-else>
                          WAL is ahead of durable storage.<br/>Pageserver is awaiting checkpoint.
                        </template>
                      </p>
                    </div>
                  </HoverCardContent>
                </HoverCard>
              </TableCell>
              <TableCell class="w-28 text-muted-foreground">{{ formatDate(branch.created_at) }}</TableCell>
              <TableCell>
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon" class="size-7 cursor-pointer">
                      <MoreVertical class="size-4"/>
                      <span class="sr-only">Open menu</span>
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" class="w-64">
                    <DropdownMenuItem
                      v-if="branch.endpoint_status === 'stopped' || branch.endpoint_status === 'failed'"
                      @click="branchStore.startEndpoint(organizationStore.selectedOrganizationId!, projectId, branch.id)"
                    >
                      <Play class="size-4"/>
                      Start
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      v-if="branch.endpoint_status === 'running'"
                      @click="branchStore.stopEndpoint(organizationStore.selectedOrganizationId!, projectId, branch.id)"
                    >
                      <Square class="size-4"/>
                      Stop
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      v-if="branch.endpoint_status === 'starting' || branch.endpoint_status === 'stopping'"
                      disabled
                    >
                      <Loader2 class="size-4 animate-spin"/>
                      {{ branch.endpoint_status === 'starting' ? 'Starting…' : 'Stopping…' }}
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      :disabled="!branch.connection_string"
                      @click="copyConnectionString(branch)"
                    >
                      <Copy class="size-4"/>
                      Copy connection string
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="openRename(branch)">
                      <Pencil class="size-4"/>
                      Rename
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="openBranchFrom(branch)">
                      <GitBranchPlus class="size-4"/>
                      Branch from here
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      class="text-destructive focus:text-destructive"
                      @click="openDelete(branch.id)"
                    >
                      <Trash2 class="size-4"/>
                      Delete
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </div>

    <AlertDialog v-model:open="deleteOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete branch?</AlertDialogTitle>
          <AlertDialogDescription>
            This branch and all its data will be permanently removed. This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="deleting">Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90 cursor-pointer"
            :disabled="deleting"
            @click="confirmDelete"
          >
            <Loader2 v-if="deleting" class="mr-1.5 size-3.5 animate-spin"/>
            Delete branch
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <Dialog v-model:open="renameOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Rename branch</DialogTitle>
        </DialogHeader>
        <form @submit.prevent="submitRename">
          <div class="grid gap-2 py-2">
            <Label for="rename-branch-name">Name</Label>
            <Input id="rename-branch-name" v-model="renameName"/>
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="renameOpen = false">Cancel</Button>
            <Button
              type="submit"
              class="cursor-pointer"
              :disabled="renameSubmitting || !renameName.trim() || renameName.trim() === renameBranch?.name"
            >
              <Loader2 v-if="renameSubmitting" class="mr-1.5 size-3.5 animate-spin"/>
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="branchFromOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Branch from {{ branchFromParent?.name }}</DialogTitle>
        </DialogHeader>
        <form @submit.prevent="submitBranchFrom">
          <div class="grid gap-2 py-2">
            <Label for="branch-from-name">New branch name</Label>
            <Input id="branch-from-name" v-model="branchFromName" placeholder="my-branch"/>
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="branchFromOpen = false">Cancel
            </Button>
            <Button type="submit" class="cursor-pointer" :disabled="branchFromSubmitting || !branchFromName.trim()">
              <Loader2 v-if="branchFromSubmitting" class="mr-1.5 size-3.5 animate-spin"/>
              Create branch
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="createOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New branch</DialogTitle>
        </DialogHeader>
        <form @submit.prevent="submitCreate">
          <div class="grid gap-2 py-2">
            <Label for="new-branch-name">Name</Label>
            <Input id="new-branch-name" v-model="createName" placeholder="my-branch"/>
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="createOpen = false">Cancel</Button>
            <Button type="submit" class="cursor-pointer" :disabled="createSubmitting || !createName.trim()">
              <Loader2 v-if="createSubmitting" class="mr-1.5 size-3.5 animate-spin"/>
              Create
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  </div>
</template>

<style>

@keyframes dash {
  from {
    background-position: 0 0;
  }
  to {
    background-position: 12px 0;
  }
}
</style>