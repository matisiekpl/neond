<script setup lang="ts">
import {ref, computed, watch} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import {useTitle} from '@vueuse/core'
import {toast} from 'vue-sonner'
import {
  BookDown, Check, Cloud, Copy, GitBranchPlus, History, KeyRound, Loader2, MoreVertical, Pencil, Play, Plug, RotateCcw, Scissors, Square, Trash2,
} from 'lucide-vue-next'
import {useProjectStore} from '@/stores/project.store'
import {useOrganizationStore} from '@/stores/organization.store'
import {useBranchStore} from '@/stores/branch.store'
import type {Branch} from '@/types/models/branch'
import EndpointStatusBadge from '@/elements/EndpointStatusBadge.vue'
import DurabilityStatusBadge from '@/elements/DurabilityStatusBadge.vue'
import RestorePitrDialog from '@/elements/RestorePitrDialog.vue'
import ConnectDialog from '@/elements/ConnectDialog.vue'
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
import {formatBytes} from "@/lib/utils.ts";

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const organizationStore = useOrganizationStore()
const branchStore = useBranchStore()

const projectId = computed(() => route.params.projectId as string)

const project = computed(() => projectStore.projects.find((p) => p.id === projectId.value))

useTitle(computed(() => project.value ? `${project.value.name} — neond` : 'neond'))

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
  const byCreatedAt = (a: BranchNode, b: BranchNode) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  roots.sort(byCreatedAt)
  for (const node of map.values()) node.children.sort(byCreatedAt)
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

function formatDate(d: string) {
  return new Date(d).toLocaleDateString('en-US', {year: 'numeric', month: 'short', day: 'numeric'})
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

const passwordOpen = ref(false)
const passwordBranch = ref<Branch | null>(null)
const passwordValue = ref('')
const passwordSubmitting = ref(false)

watch(createOpen, (val) => {
  if (!val) createName.value = ''
})
watch(branchFromOpen, (val) => {
  if (!val) branchFromName.value = ''
})
watch(renameOpen, (val) => {
  if (!val) renameName.value = ''
})
watch(passwordOpen, (val) => {
  if (!val) passwordValue.value = ''
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

function openChangePassword(branch: Branch) {
  passwordBranch.value = branch
  passwordValue.value = ''
  passwordOpen.value = true
}

const restoreOpen = ref(false)
const restoreBranch = ref<Branch | null>(null)

function openRestore(branch: Branch) {
  restoreBranch.value = branch
  restoreOpen.value = true
}

const resetOpen = ref(false)
const resetBranch = ref<Branch | null>(null)
const resetting = ref(false)

function openResetToParent(branch: Branch) {
  resetBranch.value = branch
  resetOpen.value = true
}

const detachOpen = ref(false)
const detachBranch = ref<Branch | null>(null)

function openDetach(branch: Branch) {
  detachBranch.value = branch
  detachOpen.value = true
}

async function submitCreate() {
  if (!organizationStore.selectedOrganizationId || !projectId.value) return
  const trimmed = createName.value.trim()
  if (!trimmed) return
  createSubmitting.value = true
  try {
    await branchStore.create(organizationStore.selectedOrganizationId, projectId.value, trimmed)
    createOpen.value = false
  } catch {
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
    await branchStore.create(organizationStore.selectedOrganizationId, projectId.value, trimmed, branchFromParent.value.id)
    branchFromOpen.value = false
  } catch {
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
    await branchStore.update(organizationStore.selectedOrganizationId, projectId.value, renameBranch.value.id, trimmed)
    renameOpen.value = false
  } catch {
  } finally {
    renameSubmitting.value = false
  }
}

async function submitChangePassword() {
  if (!organizationStore.selectedOrganizationId || !projectId.value || !passwordBranch.value) return
  const trimmed = passwordValue.value.trim()
  if (!trimmed) return
  passwordSubmitting.value = true
  try {
    await branchStore.changePassword(organizationStore.selectedOrganizationId, projectId.value, passwordBranch.value.id, trimmed)
    passwordOpen.value = false
  } catch {
  } finally {
    passwordSubmitting.value = false
  }
}

async function confirmResetToParent() {
  if (!organizationStore.selectedOrganizationId || !projectId.value || !resetBranch.value) return
  resetting.value = true
  try {
    await branchStore.resetToParent(organizationStore.selectedOrganizationId, projectId.value, resetBranch.value.id)
    resetOpen.value = false
  } finally {
    resetting.value = false
  }
}

async function confirmDetach() {
  if (!organizationStore.selectedOrganizationId || !projectId.value || !detachBranch.value) return
  try {
    await branchStore.detachAncestor(organizationStore.selectedOrganizationId, projectId.value, detachBranch.value.id)
    detachOpen.value = false
  } catch {
  }
}

async function confirmDelete() {
  if (!organizationStore.selectedOrganizationId || !projectId.value || !deleteId.value) return
  deleting.value = true
  try {
    await branchStore.remove(organizationStore.selectedOrganizationId, projectId.value, deleteId.value)
    deleteOpen.value = false
  } finally {
    deleting.value = false
  }
}

watch(() => organizationStore.selectedOrganizationId, (orgId) => {
  if (orgId) projectStore.fetch(orgId)
}, {immediate: true})

const connectOpen = ref(false)
const connectBranch = ref<Branch | null>(null)
const connectPooled = ref(true)
const connectLibcompat = ref(false)

function openConnect(branch: Branch) {
  connectBranch.value = branch
  connectOpen.value = true
}

const projectIdCopied = ref(false)

async function copyProjectId() {
  if (!project.value) return
  await navigator.clipboard.writeText(project.value.id)
  projectIdCopied.value = true
  toast.success('Project ID copied')
  setTimeout(() => { projectIdCopied.value = false }, 1500)
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
      @click="router.push({ name: 'projects.list', params: { organizationId: organizationStore.selectedOrganizationId } })"
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
        <span>Created {{ new Date(project.created_at).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' }) }}</span>
        <template v-if="project.size !== undefined">
          <span>·</span>
          <span>{{ formatBytes(project.size) }}</span>
        </template>
        <button
          type="button"
          class="group inline-flex cursor-pointer items-center gap-1.5 rounded border border-border bg-muted/50 px-2 py-0.5 font-mono text-xs transition-colors hover:bg-muted"
          :title="`Copy project ID: ${project.id}`"
          @click="copyProjectId"
        >
          <span class="text-[10px] uppercase tracking-wider text-muted-foreground/70">ID</span>
          <span class="break-all">{{ project.id }}</span>
          <Check v-if="projectIdCopied" class="size-3 text-emerald-500" />
          <Copy v-else class="size-3 opacity-50 group-hover:opacity-100" />
        </button>
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
            <TableRow
              v-for="{ branch, depth } in rows"
              :key="branch.id"
              class="cursor-pointer"
              @click="router.push({ name: 'projects.branches.data', params: { organizationId: organizationStore.selectedOrganizationId!, projectId, branchId: branch.id } })"
            >
              <TableCell class="font-medium">
                <span class="flex items-center gap-1.5" :style="{ paddingLeft: `${depth * 20}px` }">
                  <span v-if="depth > 0" class="shrink-0 text-muted-foreground">╰</span>
                  <span>{{ branch.name }}</span>
                </span>
              </TableCell>
              <TableCell class="w-28">
                <EndpointStatusBadge :status="branch.endpoint_status"/>
              </TableCell>
              <TableCell class="w-24 text-xs text-muted-foreground">
                {{ formatBytes(branch.current_logical_size) }}
              </TableCell>
              <TableCell class="w-44" @click.stop>
                <HoverCard :open-delay="0" :close-delay="500">
                  <HoverCardTrigger as-child>
                    <DurabilityStatusBadge
                      :last-record-lsn="branch.last_record_lsn"
                      :remote-consistent-lsn="branch.remote_consistent_lsn_visible"
                    />
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
              <TableCell @click.stop>
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
                      @click="branchStore.launchEndpoint(organizationStore.selectedOrganizationId!, projectId, branch.id)"
                    >
                      <Play class="size-4"/>
                      Start
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      v-if="branch.endpoint_status === 'running'"
                      @click="branchStore.shutdownEndpoint(organizationStore.selectedOrganizationId!, projectId, branch.id)"
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
                      :disabled="!branch.connection_string && !branch.pooler_connection_string"
                      @click="openConnect(branch)"
                    >
                      <Plug class="size-4"/>
                      Connect
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="openRename(branch)">
                      <Pencil class="size-4"/>
                      Rename
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="openChangePassword(branch)">
                      <KeyRound class="size-4"/>
                      Change password
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="openBranchFrom(branch)">
                      <GitBranchPlus class="size-4"/>
                      Branch from here
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="openRestore(branch)">
                      <History class="size-4"/>
                      Restore from PITR
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      v-if="branch.parent_branch_id"
                      @click="openResetToParent(branch)"
                    >
                      <RotateCcw class="size-4"/>
                      Reset to parent
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      v-if="branch.parent_branch_id"
                      @click="openDetach(branch)"
                    >
                      <Scissors class="size-4"/>
                      Detach from ancestor
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

    <AlertDialog
      :open="detachOpen"
      @update:open="(value: boolean) => { if (!branchStore.detaching) detachOpen = value }"
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Detach from ancestor?</AlertDialogTitle>
          <AlertDialogDescription>
            <b>{{ detachBranch?.name }}</b> will be permanently separated from its parent. Its data will be copied so it no longer depends on the parent branch. This may take a while and cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="branchStore.detaching">Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="cursor-pointer"
            :disabled="branchStore.detaching"
            @click="confirmDetach"
          >
            <Loader2 v-if="branchStore.detaching" class="mr-1.5 size-3.5 animate-spin"/>
            Detach
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <AlertDialog v-model:open="resetOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Reset branch to parent?</AlertDialogTitle>
          <AlertDialogDescription>
            All data on <b>{{ resetBranch?.name }}</b> will be replaced with the current state of its parent. The endpoint will briefly restart. This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="resetting">Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90 cursor-pointer"
            :disabled="resetting"
            @click="confirmResetToParent"
          >
            <Loader2 v-if="resetting" class="mr-1.5 size-3.5 animate-spin"/>
            Reset branch
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

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

    <Dialog v-model:open="passwordOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Change password</DialogTitle>
        </DialogHeader>
        <form @submit.prevent="submitChangePassword">
          <div class="grid gap-2 py-2">
            <Label for="change-password">New password</Label>
            <Input id="change-password" v-model="passwordValue" type="password" autocomplete="new-password"/>
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="passwordOpen = false">Cancel</Button>
            <Button
              type="submit"
              class="cursor-pointer"
              :disabled="passwordSubmitting || !passwordValue.trim()"
            >
              <Loader2 v-if="passwordSubmitting" class="mr-1.5 size-3.5 animate-spin"/>
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

    <RestorePitrDialog
      v-if="organizationStore.selectedOrganizationId && restoreBranch"
      v-model:open="restoreOpen"
      :organization-id="organizationStore.selectedOrganizationId"
      :project-id="projectId"
      :branch-id="restoreBranch.id"
    />

    <ConnectDialog
      v-model:open="connectOpen"
      v-model:pooled="connectPooled"
      v-model:libcompat="connectLibcompat"
      :branch="connectBranch"
    />
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