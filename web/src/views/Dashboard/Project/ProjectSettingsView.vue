<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTitle } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { Loader2 } from 'lucide-vue-next'
import { useProjectStore } from '@/stores/project.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { projectsApi } from '@/api/projects'
import { getAppError } from '@/api/utils'
import MegabytesInput from '@/elements/MegabytesInput.vue'
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
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Slider } from '@/components/ui/slider'

const PITR_PRESETS = [
  { label: '0h', value: '' },
  { label: '1h', value: '1h' },
  { label: '6h', value: '6h' },
  { label: '12h', value: '12h' },
  { label: '1 day', value: '1day' },
  { label: '3 days', value: '3days' },
  { label: '7 days', value: '7days' },
  { label: '14 days', value: '14days' },
  { label: '30 days', value: '30days' },
]

const GC_PERIOD_PRESETS = [
  { label: '10m', value: '10m' },
  { label: '30m', value: '30m' },
  { label: '1h', value: '1h' },
  { label: '2h', value: '2h' },
  { label: '6h', value: '6h' },
  { label: '12h', value: '12h' },
  { label: '24h', value: '1day' },
]

const CHECKPOINT_TIMEOUT_PRESETS = [
  { label: '1m', value: '1m' },
  { label: '5m', value: '5m' },
  { label: '10m', value: '10m' },
  { label: '30m', value: '30m' },
  { label: '1h', value: '1h' },
]

function presetToIndex(presets: { value: string }[], v: string) {
  const idx = presets.findIndex((p) => p.value === v)
  return idx === -1 ? 0 : idx
}

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const organizationStore = useOrganizationStore()

const projectId = computed(() => route.params.projectId as string)
const project = computed(() => projectStore.projects.find((p) => p.id === projectId.value))

useTitle(computed(() => project.value ? `Settings — ${project.value.name} — neond` : 'Settings — neond'))

const deleteOpen = ref(false)
const deleting = ref(false)
const configLoading = ref(false)
const nameSubmitting = ref(false)
const gcSubmitting = ref(false)
const pitrSubmitting = ref(false)
const checkpointSubmitting = ref(false)

const form = reactive({
  name: '',
  gcPeriod: '',
  gcHorizon: '',
  pitrInterval: '7days',
  checkpointDistance: '',
  checkpointTimeout: '',
})

const saved = ref({ ...form })

function loadSaved(s: typeof saved.value) {
  Object.assign(form, s)
  saved.value = { ...s }
}

const gcPeriodIndex = computed({
  get: () => [presetToIndex(GC_PERIOD_PRESETS, form.gcPeriod)],
  set: (v: number[]) => { form.gcPeriod = GC_PERIOD_PRESETS[v[0]]?.value ?? '' },
})

const pitrIndex = computed({
  get: () => [presetToIndex(PITR_PRESETS, form.pitrInterval)],
  set: (v: number[]) => { form.pitrInterval = PITR_PRESETS[v[0]]?.value ?? '' },
})

const checkpointTimeoutIndex = computed({
  get: () => [presetToIndex(CHECKPOINT_TIMEOUT_PRESETS, form.checkpointTimeout)],
  set: (v: number[]) => { form.checkpointTimeout = CHECKPOINT_TIMEOUT_PRESETS[v[0]]?.value ?? '' },
})

const nameIsDirty = computed(() => form.name.trim() !== saved.value.name)
const gcIsDirty = computed(() => form.gcPeriod !== saved.value.gcPeriod || form.gcHorizon !== saved.value.gcHorizon)
const pitrIsDirty = computed(() => form.pitrInterval !== saved.value.pitrInterval)
const checkpointIsDirty = computed(() => form.checkpointDistance !== saved.value.checkpointDistance || form.checkpointTimeout !== saved.value.checkpointTimeout)

watch(() => organizationStore.selectedOrganizationId, (orgId) => {
  if (orgId) projectStore.fetch(orgId)
}, { immediate: true })

watch(project, (p) => {
  if (p) form.name = p.name
}, { immediate: true })

watch(
  [() => organizationStore.selectedOrganizationId, projectId],
  async ([orgId, pid]) => {
    if (!orgId || !pid) return
    configLoading.value = true
    try {
      const p = await projectsApi.get(orgId, pid)
      loadSaved({
        name: p.name,
        gcPeriod: p.gc_period ?? '',
        gcHorizon: p.gc_horizon !== undefined ? String(p.gc_horizon) : '',
        pitrInterval: p.pitr_interval ?? '7days',
        checkpointDistance: p.checkpoint_distance !== undefined ? String(p.checkpoint_distance) : '',
        checkpointTimeout: p.checkpoint_timeout ?? '',
      })
    } catch {}
    finally {
      configLoading.value = false
    }
  },
  { immediate: true },
)

async function saveName() {
  if (!organizationStore.selectedOrganizationId || !projectId.value) return
  const trimmed = form.name.trim()
  if (!trimmed) return
  nameSubmitting.value = true
  try {
    await projectStore.update(organizationStore.selectedOrganizationId, projectId.value, { name: trimmed })
    saved.value = { ...saved.value, name: trimmed }
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    nameSubmitting.value = false
  }
}

async function saveGc() {
  if (!organizationStore.selectedOrganizationId || !projectId.value) return
  gcSubmitting.value = true
  try {
    await projectStore.update(organizationStore.selectedOrganizationId, projectId.value, {
      gc_period: form.gcPeriod.trim() || undefined,
      gc_horizon: form.gcHorizon.trim() ? Number(form.gcHorizon) : undefined,
    })
    saved.value = { ...saved.value, gcPeriod: form.gcPeriod, gcHorizon: form.gcHorizon }
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    gcSubmitting.value = false
  }
}

async function savePitr() {
  if (!organizationStore.selectedOrganizationId || !projectId.value) return
  pitrSubmitting.value = true
  try {
    await projectStore.update(organizationStore.selectedOrganizationId, projectId.value, {
      pitr_interval: form.pitrInterval.trim() || undefined,
    })
    saved.value = { ...saved.value, pitrInterval: form.pitrInterval }
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    pitrSubmitting.value = false
  }
}

async function saveCheckpoint() {
  if (!organizationStore.selectedOrganizationId || !projectId.value) return
  checkpointSubmitting.value = true
  try {
    await projectStore.update(organizationStore.selectedOrganizationId, projectId.value, {
      checkpoint_distance: form.checkpointDistance.trim() ? Number(form.checkpointDistance) : undefined,
      checkpoint_timeout: form.checkpointTimeout.trim() || undefined,
    })
    saved.value = { ...saved.value, checkpointDistance: form.checkpointDistance, checkpointTimeout: form.checkpointTimeout }
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    checkpointSubmitting.value = false
  }
}

async function confirmDelete() {
  if (!organizationStore.selectedOrganizationId || !projectId.value) return
  deleting.value = true
  try {
    await projectStore.remove(organizationStore.selectedOrganizationId, projectId.value)
    await router.push({ name: 'projects.list', params: { organizationId: organizationStore.selectedOrganizationId } })
  } catch {
    deleting.value = false
  }
}
</script>

<template>
  <div v-if="projectStore.loading" class="flex justify-center py-12">
    <Loader2 class="size-6 animate-spin" />
  </div>

  <div
    v-else-if="!project"
    class="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center"
  >
    <p class="text-sm font-medium">Project not found</p>
    <button
      type="button"
      class="mt-4 text-sm underline underline-offset-4"
      @click="router.push({ name: 'projects.list', params: { organizationId: organizationStore.selectedOrganizationId } })"
    >
      Back to projects
    </button>
  </div>

  <div v-else class="w-full divide-y">
    <section class="flex flex-col gap-4 py-8 md:grid md:grid-cols-[280px_1fr] md:gap-8">
      <div>
        <h3 class="text-sm font-medium">General</h3>
        <p class="mt-1 text-sm text-muted-foreground">Update your project name.</p>
      </div>
      <form @submit.prevent="saveName" class="space-y-2 max-w-sm">
        <Label for="project-name">Name</Label>
        <Input id="project-name" v-model="form.name" />
        <Button type="submit" class="mt-2 cursor-pointer" :disabled="nameSubmitting || !form.name.trim() || !nameIsDirty">
          <Loader2 v-if="nameSubmitting" class="mr-1.5 size-3.5 animate-spin" />
          Save
        </Button>
      </form>
    </section>

    <section class="flex flex-col gap-4 py-8 md:grid md:grid-cols-[280px_1fr] md:gap-8">
      <div>
        <h3 class="text-sm font-medium">Garbage collection</h3>
        <p class="mt-1 text-sm text-muted-foreground">Control when old data versions are removed.</p>
      </div>
      <div>
        <div v-if="configLoading" class="flex justify-center py-4">
          <Loader2 class="size-5 animate-spin" />
        </div>
        <form v-else @submit.prevent="saveGc">
          <div class="flex items-start gap-6">
            <div class="grid gap-2 w-36">
              <Label for="gc-horizon">GC horizon</Label>
              <MegabytesInput id="gc-horizon" v-model="form.gcHorizon" />
              <p class="text-xs text-muted-foreground">WAL distance beyond which data can be GC'd.</p>
            </div>
            <div class="flex-1 grid gap-3">
              <Label>GC period</Label>
              <div class="space-y-3">
                <Slider :min="0" :max="GC_PERIOD_PRESETS.length - 1" :step="1" v-model="gcPeriodIndex" />
                <div class="relative h-4">
                  <span
                    v-for="(p, i) in GC_PERIOD_PRESETS" :key="p.label"
                    class="absolute text-xs text-muted-foreground"
                    :style="{
                      left: i === GC_PERIOD_PRESETS.length - 1 ? undefined : `${(i / (GC_PERIOD_PRESETS.length - 1)) * 100}%`,
                      right: i === GC_PERIOD_PRESETS.length - 1 ? '0%' : undefined,
                      transform: i === 0 || i === GC_PERIOD_PRESETS.length - 1 ? undefined : 'translateX(-50%)',
                    }"
                  >{{ p.label }}</span>
                </div>
              </div>
              <p class="text-xs text-muted-foreground">How often garbage collection runs.</p>
            </div>
          </div>
          <Button type="submit" class="mt-4 cursor-pointer" :disabled="gcSubmitting || !gcIsDirty">
            <Loader2 v-if="gcSubmitting" class="mr-1.5 size-3.5 animate-spin" />
            Save
          </Button>
        </form>
      </div>
    </section>

    <section class="flex flex-col gap-4 py-8 md:grid md:grid-cols-[280px_1fr] md:gap-8">
      <div>
        <h3 class="text-sm font-medium">Point-in-time recovery</h3>
        <p class="mt-1 text-sm text-muted-foreground">
          Choose the length of your restore window. Enables
          <strong>instant restore</strong> for point-in-time recovery, time travel queries, and branching from past states.
        </p>
      </div>
      <div>
        <div v-if="configLoading" class="flex justify-center py-4">
          <Loader2 class="size-5 animate-spin" />
        </div>
        <form v-else @submit.prevent="savePitr" class="space-y-6">
          <div class="space-y-3">
            <Slider :min="0" :max="PITR_PRESETS.length - 1" :step="1" v-model="pitrIndex" />
            <div class="relative h-4">
              <span
                v-for="(p, i) in PITR_PRESETS" :key="p.label"
                class="absolute text-xs text-muted-foreground"
                :style="{
                  left: i === PITR_PRESETS.length - 1 ? undefined : `${(i / (PITR_PRESETS.length - 1)) * 100}%`,
                  right: i === PITR_PRESETS.length - 1 ? '0%' : undefined,
                  transform: i === 0 || i === PITR_PRESETS.length - 1 ? undefined : 'translateX(-50%)',
                }"
              >{{ p.label }}</span>
            </div>
          </div>
          <Button type="submit" class="cursor-pointer" :disabled="pitrSubmitting || !pitrIsDirty">
            <Loader2 v-if="pitrSubmitting" class="mr-1.5 size-3.5 animate-spin" />
            Save
          </Button>
        </form>
      </div>
    </section>

    <section class="flex flex-col gap-4 py-8 md:grid md:grid-cols-[280px_1fr] md:gap-8">
      <div>
        <h3 class="text-sm font-medium">Checkpointing</h3>
        <p class="mt-1 text-sm text-muted-foreground">Tune how frequently the pageserver flushes data to disk.</p>
      </div>
      <div>
        <div v-if="configLoading" class="flex justify-center py-4">
          <Loader2 class="size-5 animate-spin" />
        </div>
        <form v-else @submit.prevent="saveCheckpoint">
          <div class="flex items-start gap-6">
            <div class="grid gap-2 w-36">
              <Label for="checkpoint-distance">Checkpoint distance</Label>
              <MegabytesInput id="checkpoint-distance" v-model="form.checkpointDistance" />
              <p class="text-xs text-muted-foreground">Amount of WAL data between checkpoints.</p>
            </div>
            <div class="flex-1 grid gap-3">
              <Label>Checkpoint timeout</Label>
              <div class="space-y-3">
                <Slider :min="0" :max="CHECKPOINT_TIMEOUT_PRESETS.length - 1" :step="1" v-model="checkpointTimeoutIndex" />
                <div class="relative h-4">
                  <span
                    v-for="(p, i) in CHECKPOINT_TIMEOUT_PRESETS" :key="p.label"
                    class="absolute text-xs text-muted-foreground"
                    :style="{
                      left: i === CHECKPOINT_TIMEOUT_PRESETS.length - 1 ? undefined : `${(i / (CHECKPOINT_TIMEOUT_PRESETS.length - 1)) * 100}%`,
                      right: i === CHECKPOINT_TIMEOUT_PRESETS.length - 1 ? '0%' : undefined,
                      transform: i === 0 || i === CHECKPOINT_TIMEOUT_PRESETS.length - 1 ? undefined : 'translateX(-50%)',
                    }"
                  >{{ p.label }}</span>
                </div>
              </div>
              <p class="text-xs text-muted-foreground">Maximum time between forced checkpoints.</p>
            </div>
          </div>
          <Button type="submit" class="mt-4 cursor-pointer" :disabled="checkpointSubmitting || !checkpointIsDirty">
            <Loader2 v-if="checkpointSubmitting" class="mr-1.5 size-3.5 animate-spin" />
            Save
          </Button>
        </form>
      </div>
    </section>

    <section class="flex flex-col gap-4 py-8 md:grid md:grid-cols-[280px_1fr] md:gap-8">
      <div>
        <h3 class="text-sm font-medium text-destructive">Danger zone</h3>
        <p class="mt-1 text-sm text-muted-foreground">Irreversible actions that affect this project.</p>
      </div>
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">Delete project</p>
          <p class="text-sm text-muted-foreground">Permanently remove this project and all its data.</p>
        </div>
        <Button variant="destructive" type="button" class="cursor-pointer" @click="deleteOpen = true">
          Delete project
        </Button>
      </div>
    </section>

    <AlertDialog v-model:open="deleteOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete project?</AlertDialogTitle>
          <AlertDialogDescription>
            All branches and data in <strong>{{ project.name }}</strong> will be permanently removed. This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="deleting">Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90 cursor-pointer"
            :disabled="deleting"
            @click="confirmDelete"
          >
            <Loader2 v-if="deleting" class="mr-1.5 size-3.5 animate-spin" />
            Delete project
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>