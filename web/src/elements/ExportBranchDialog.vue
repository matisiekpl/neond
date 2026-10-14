<script setup lang="ts">
import {computed, onUnmounted, ref, watch} from 'vue'
import {Check, CircleAlert, Copy, Download, FileArchive, FileCode, Loader2, Trash2} from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {Button} from '@/components/ui/button'
import {Checkbox} from '@/components/ui/checkbox'
import {Input} from '@/components/ui/input'
import {Label} from '@/components/ui/label'
import {Progress} from '@/components/ui/progress'
import {sqlApi} from '@/api/sql'
import {useBranchStore} from '@/stores/branch.store'
import {formatBytes} from '@/lib/utils'
import type {Branch} from '@/types/models/branch'
import {isExportActive, pgMajorVersion, type ExportFormat, type ExportStatus} from '@/types/dto/export'

const props = defineProps<{
  open: boolean
  organizationId: string
  projectId: string
  branch: Branch
  pgVersion: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'status': [status: ExportStatus]
}>()

const STATUS_POLL_INTERVAL_MS = 1000
const DATABASES_SQL = 'SELECT datname FROM pg_database WHERE NOT datistemplate AND datallowconn ORDER BY 1'

const branchStore = useBranchStore()

const status = ref<ExportStatus | null>(null)
const showForm = ref(false)
const format = ref<ExportFormat>('custom')
const database = ref('postgres')
const databases = ref<string[]>([])
const databasesLoading = ref(false)
const noOwner = ref(false)
const noPrivileges = ref(false)
const starting = ref(false)
const cancelling = ref(false)
const downloading = ref(false)
const now = ref(Date.now())
const restoreCommandCopied = ref(false)
let pollTimer: ReturnType<typeof setInterval> | null = null

const active = computed(() => isExportActive(status.value?.state))
const view = computed<'loading' | 'form' | 'progress' | 'done'>(() => {
  if (!status.value) return 'loading'
  if (active.value) return 'progress'
  if (status.value.state === 'completed' && !showForm.value) return 'done'
  return 'form'
})

const majorVersion = computed(() => pgMajorVersion(status.value?.pg_version ?? props.pgVersion))
const percent = computed(() => Math.round((status.value?.progress ?? 0) * 100))

const phaseLabel = computed(() => {
  const current = status.value
  if (!current) return ''
  switch (current.state) {
    case 'preparing':
      return props.branch.endpoint_status === 'running'
        ? 'Estimating the size of the database…'
        : 'Starting the compute and estimating the size of the database…'
    case 'dumping':
      return current.current_table ? `Dumping ${current.current_table}` : 'Dumping schema…'
    case 'finalizing':
      return 'Writing the file to disk…'
    default:
      return ''
  }
})

const elapsed = computed(() => {
  const startedAt = status.value?.started_at
  if (!startedAt) return null
  const end = status.value?.finished_at ? new Date(status.value.finished_at).getTime() : now.value
  const seconds = Math.max(0, Math.round((end - new Date(startedAt).getTime()) / 1000))
  const minutes = Math.floor(seconds / 60)
  return minutes > 0 ? `${minutes}m ${seconds % 60}s` : `${seconds}s`
})

const expiresAt = computed(() => {
  const value = status.value?.expires_at
  return value ? new Date(value).toLocaleTimeString('en-US', {hour: '2-digit', minute: '2-digit'}) : null
})

const restoreCommand = computed(() => {
  const fileName = status.value?.file_name ?? 'export'
  return status.value?.format === 'custom'
    ? `pg_restore --no-owner -d "$DATABASE_URL" ${fileName}`
    : `psql "$DATABASE_URL" -f ${fileName}`
})

const logsRoute = computed(() => ({
  name: 'projects.branches.logs',
  params: {
    organizationId: props.organizationId,
    projectId: props.projectId,
    branchId: props.branch.id,
    component: 'export',
  },
}))

function setStatus(value: ExportStatus) {
  status.value = value
  emit('status', value)
  if (isExportActive(value.state)) startPolling()
  else stopPolling()
}

async function fetchStatus() {
  now.value = Date.now()
  try {
    setStatus(await branchStore.exportStatus(props.organizationId, props.projectId, props.branch.id))
  } catch {
  }
}

function startPolling() {
  if (!pollTimer) pollTimer = setInterval(fetchStatus, STATUS_POLL_INTERVAL_MS)
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

async function loadDatabases() {
  databases.value = []
  if (props.branch.endpoint_status !== 'running') return
  databasesLoading.value = true
  try {
    const response = await sqlApi.execute(props.organizationId, props.projectId, props.branch.id, DATABASES_SQL)
    if (!response.error) {
      databases.value = response.rows.map((row) => row[0]).filter((name): name is string => !!name)
      if (databases.value.length > 0 && !databases.value.includes(database.value)) {
        database.value = databases.value.includes('postgres') ? 'postgres' : (databases.value[0] ?? 'postgres')
      }
    }
  } catch {
  } finally {
    databasesLoading.value = false
  }
}

watch(
  [() => props.open, () => props.branch.id],
  ([open]) => {
    stopPolling()
    if (!open) return
    status.value = null
    showForm.value = false
    restoreCommandCopied.value = false
    fetchStatus()
    loadDatabases()
  },
  {immediate: true},
)

onUnmounted(stopPolling)

async function submit() {
  const name = database.value.trim() || 'postgres'
  starting.value = true
  try {
    const started = await branchStore.startExport(props.organizationId, props.projectId, props.branch.id, {
      database: name,
      format: format.value,
      no_owner: noOwner.value,
      no_privileges: noPrivileges.value,
    })
    showForm.value = false
    setStatus(started)
  } catch {
  } finally {
    starting.value = false
  }
}

async function cancelOrDiscard() {
  cancelling.value = true
  try {
    setStatus(await branchStore.cancelExport(props.organizationId, props.projectId, props.branch.id))
    if (active.value) startPolling()
  } catch {
  } finally {
    cancelling.value = false
  }
}

async function download() {
  downloading.value = true
  try {
    await branchStore.downloadExport(props.organizationId, props.projectId, props.branch.id)
  } catch {
  } finally {
    downloading.value = false
  }
}

async function copyRestoreCommand() {
  await navigator.clipboard.writeText(restoreCommand.value)
  restoreCommandCopied.value = true
  setTimeout(() => (restoreCommandCopied.value = false), 1500)
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>Export {{ branch.name }}</DialogTitle>
        <DialogDescription>
          Dump the branch with pg_dump<template v-if="majorVersion"> {{ majorVersion }}</template>, the version of
          this project's Postgres. The dump restores on PostgreSQL {{ majorVersion ?? '' }} or newer.
        </DialogDescription>
      </DialogHeader>

      <div v-if="view === 'loading'" class="flex h-32 items-center justify-center">
        <Loader2 class="size-5 animate-spin text-muted-foreground"/>
      </div>

      <form v-else-if="view === 'form'" id="export-form" class="flex flex-col gap-4" @submit.prevent="submit">
        <div
          v-if="status?.state === 'failed'"
          class="flex gap-2 border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700 dark:border-red-900/60 dark:bg-red-950/40 dark:text-red-300"
        >
          <CircleAlert class="mt-0.5 size-3.5 shrink-0"/>
          <div class="min-w-0">
            <p class="font-medium">The last export failed</p>
            <p class="mt-0.5 whitespace-pre-wrap break-words font-mono text-[11px]">{{ status.error }}</p>
            <RouterLink :to="logsRoute" class="mt-1 inline-block underline" @click="emit('update:open', false)">
              Show export logs
            </RouterLink>
          </div>
        </div>
        <p v-else-if="status?.state === 'cancelled'" class="text-xs text-muted-foreground">
          The last export was cancelled.
        </p>

        <div class="flex flex-col gap-2">
          <Label>Format</Label>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              class="flex cursor-pointer flex-col gap-1 border px-3 py-2 text-left text-xs transition-colors"
              :class="format === 'custom' ? 'border-primary bg-primary/5' : 'border-border hover:bg-muted/50'"
              data-testid="export-format-custom"
              @click="format = 'custom'"
            >
              <span class="flex items-center gap-1.5 font-medium"><FileArchive class="size-3.5"/>Custom (.dump)</span>
              <span class="text-muted-foreground">Compressed archive, restored with pg_restore</span>
            </button>
            <button
              type="button"
              class="flex cursor-pointer flex-col gap-1 border px-3 py-2 text-left text-xs transition-colors"
              :class="format === 'plain' ? 'border-primary bg-primary/5' : 'border-border hover:bg-muted/50'"
              data-testid="export-format-plain"
              @click="format = 'plain'"
            >
              <span class="flex items-center gap-1.5 font-medium"><FileCode class="size-3.5"/>Plain SQL (.sql)</span>
              <span class="text-muted-foreground">Readable SQL script, restored with psql</span>
            </button>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <Label for="export-database">Database</Label>
          <select
            v-if="databases.length > 0"
            id="export-database"
            v-model="database"
            class="flex h-9 w-full border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          >
            <option v-for="name in databases" :key="name" :value="name">{{ name }}</option>
          </select>
          <template v-else>
            <div class="relative">
              <Input id="export-database" v-model="database" placeholder="postgres" autocomplete="off"/>
              <Loader2 v-if="databasesLoading" class="absolute right-2 top-1/2 size-3.5 -translate-y-1/2 animate-spin text-muted-foreground"/>
            </div>
            <span v-if="branch.endpoint_status !== 'running'" class="text-xs text-muted-foreground">
              The compute of this branch is stopped; it will be started for the export.
            </span>
          </template>
        </div>

        <div class="flex flex-col gap-2">
          <label class="flex cursor-pointer items-center gap-2 text-xs">
            <Checkbox :model-value="noOwner" @update:model-value="noOwner = $event === true"/>
            Skip ownership <span class="font-mono text-muted-foreground">--no-owner</span>
          </label>
          <label class="flex cursor-pointer items-center gap-2 text-xs">
            <Checkbox :model-value="noPrivileges" @update:model-value="noPrivileges = $event === true"/>
            Skip privileges <span class="font-mono text-muted-foreground">--no-privileges</span>
          </label>
        </div>
      </form>

      <div v-else-if="view === 'progress' && status" class="flex flex-col gap-3" data-testid="export-progress">
        <div class="flex items-center justify-between gap-3 text-xs">
          <span class="flex min-w-0 items-center gap-1.5">
            <Loader2 class="size-3.5 shrink-0 animate-spin"/>
            <span class="truncate" :title="phaseLabel">{{ phaseLabel }}</span>
          </span>
          <span class="shrink-0 font-mono tabular-nums">{{ percent }}%</span>
        </div>
        <Progress
          :model-value="percent"
          :class="status.state === 'preparing' ? 'animate-pulse' : ''"
        />
        <dl class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <dt class="text-muted-foreground">Tables</dt>
          <dd class="text-right tabular-nums">{{ status.tables_done }} / {{ status.tables_total }}</dd>
          <dt class="text-muted-foreground">Written</dt>
          <dd class="text-right tabular-nums">
            {{ formatBytes(status.bytes_written) }}
            <span v-if="status.bytes_total_estimate > 0" class="text-muted-foreground">
              (table data ≈ {{ formatBytes(status.bytes_total_estimate) }})
            </span>
          </dd>
          <dt class="text-muted-foreground">Elapsed</dt>
          <dd class="text-right tabular-nums">{{ elapsed }}</dd>
          <dt class="text-muted-foreground">Database · format</dt>
          <dd class="text-right">{{ status.database }} · {{ status.format === 'custom' ? '.dump' : '.sql' }}</dd>
        </dl>
        <p class="text-xs text-muted-foreground">
          You can close this dialog; the export continues on the server and you will be notified when it is ready.
          <RouterLink :to="logsRoute" class="underline" @click="emit('update:open', false)">Show logs</RouterLink>
        </p>
      </div>

      <div v-else-if="view === 'done' && status" class="flex flex-col gap-3" data-testid="export-done">
        <div class="flex items-center gap-2 border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-800 dark:border-emerald-900/60 dark:bg-emerald-950/40 dark:text-emerald-300">
          <Check class="size-3.5 shrink-0"/>
          <span class="min-w-0 truncate font-mono" :title="status.file_name ?? ''">{{ status.file_name }}</span>
          <span class="ml-auto shrink-0 tabular-nums">{{ formatBytes(status.bytes_written) }}</span>
        </div>
        <dl class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
          <dt class="text-muted-foreground">pg_dump</dt>
          <dd class="text-right">{{ status.pg_dump_version ?? '—' }}</dd>
          <dt class="text-muted-foreground">Tables</dt>
          <dd class="text-right tabular-nums">{{ status.tables_total }}</dd>
          <dt class="text-muted-foreground">Took</dt>
          <dd class="text-right tabular-nums">{{ elapsed }}</dd>
          <dt class="text-muted-foreground">Available until</dt>
          <dd class="text-right">{{ expiresAt }}</dd>
        </dl>
        <div class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">
            Restore with {{ status.format === 'custom' ? 'pg_restore' : 'psql' }} {{ majorVersion }} or newer:
          </span>
          <button
            type="button"
            class="group flex cursor-pointer items-center gap-2 border bg-muted/50 px-2 py-1.5 text-left font-mono text-[11px] hover:bg-muted"
            @click="copyRestoreCommand"
          >
            <span class="min-w-0 flex-1 break-all">{{ restoreCommand }}</span>
            <Check v-if="restoreCommandCopied" class="size-3 shrink-0 text-emerald-500"/>
            <Copy v-else class="size-3 shrink-0 opacity-50 group-hover:opacity-100"/>
          </button>
        </div>
      </div>

      <DialogFooter class="mt-2">
        <template v-if="view === 'form'">
          <Button variant="outline" class="cursor-pointer" :disabled="starting" @click="emit('update:open', false)">
            Cancel
          </Button>
          <Button type="submit" form="export-form" class="cursor-pointer" :disabled="starting" data-testid="export-start">
            <Loader2 v-if="starting" class="mr-1.5 size-3.5 animate-spin"/>
            {{ starting ? 'Starting…' : 'Start export' }}
          </Button>
        </template>
        <template v-else-if="view === 'progress'">
          <Button variant="outline" class="cursor-pointer" @click="emit('update:open', false)">
            Close
          </Button>
          <Button variant="destructive" class="cursor-pointer" :disabled="cancelling" @click="cancelOrDiscard">
            <Loader2 v-if="cancelling" class="mr-1.5 size-3.5 animate-spin"/>
            Cancel export
          </Button>
        </template>
        <template v-else-if="view === 'done'">
          <Button variant="outline" class="cursor-pointer" :disabled="cancelling" @click="cancelOrDiscard">
            <Trash2 class="mr-1.5 size-3.5"/>
            Delete file
          </Button>
          <Button variant="outline" class="cursor-pointer" @click="showForm = true">
            New export
          </Button>
          <Button class="cursor-pointer" :disabled="downloading" data-testid="export-download" @click="download">
            <Loader2 v-if="downloading" class="mr-1.5 size-3.5 animate-spin"/>
            <Download v-else class="mr-1.5 size-3.5"/>
            Download
          </Button>
        </template>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
