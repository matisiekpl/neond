<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useTitle } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { Lightbulb, Loader2, RotateCcw, TriangleAlert } from 'lucide-vue-next'
import { useBranchStore } from '@/stores/branch.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { branchesApi } from '@/api/branches'
import { getAppError } from '@/api/utils'
import type { ComputeSettings, ComputeSettingsOverrides } from '@/types/dto/computeSettings'
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
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Progress } from '@/components/ui/progress'
import { Slider } from '@/components/ui/slider'

const SHARED_BUFFERS_STEPS = [64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536]

type OverrideKey = keyof ComputeSettingsOverrides

const OVERRIDE_FIELDS: { key: OverrideKey, label: string, unit: string, hint: string }[] = [
  { key: 'effective_cache_size_mb', label: 'effective_cache_size', unit: 'MB', hint: 'Auto: 4 × shared_buffers' },
  { key: 'maintenance_work_mem_mb', label: 'maintenance_work_mem', unit: 'MB', hint: 'Auto: shared_buffers, max 2 GB' },
  { key: 'work_mem_mb', label: 'work_mem', unit: 'MB', hint: 'Auto: shared_buffers / 16, max 64 MB' },
  { key: 'max_connections', label: 'max_connections', unit: '', hint: 'Auto: 100' },
]

// Mirrors ComputeSettings::derive in src/mgmt/compute/settings.rs, for preview only.
function derive(sharedBuffersMb: number): Record<OverrideKey, number> {
  return {
    effective_cache_size_mb: sharedBuffersMb * 4,
    maintenance_work_mem_mb: Math.min(sharedBuffersMb, 2048),
    work_mem_mb: Math.min(Math.max(Math.floor(sharedBuffersMb / 16), 1), 64),
    max_connections: 100,
  }
}

function formatMb(mb: number): string {
  if (mb >= 1024) {
    const gb = mb / 1024
    return `${Number.isInteger(gb) ? gb : gb.toFixed(1)} GB`
  }
  return `${mb} MB`
}

const route = useRoute()
const branchStore = useBranchStore()
const organizationStore = useOrganizationStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)
const branch = computed(() => branchStore.branches.find((b) => b.id === branchId.value))

useTitle(computed(() => branch.value ? `Settings — ${branch.value.name} — neond` : 'Settings — neond'))

const loading = ref(false)
const submitting = ref(false)
const restartConfirmOpen = ref(false)
const settings = ref<ComputeSettings | null>(null)

const sharedBuffersMb = ref(128)
const overrides = ref<Record<OverrideKey, string | number>>({
  effective_cache_size_mb: '',
  maintenance_work_mem_mb: '',
  work_mem_mb: '',
  max_connections: '',
})

function load(s: ComputeSettings) {
  settings.value = s
  sharedBuffersMb.value = s.shared_buffers_mb
  for (const field of OVERRIDE_FIELDS) {
    const value = s.overrides[field.key]
    overrides.value[field.key] = value === null ? '' : String(value)
  }
}

watch(
  [() => organizationStore.selectedOrganizationId, projectId, branchId],
  async ([orgId, pid, bid]) => {
    if (!orgId || !pid || !bid) return
    loading.value = true
    try {
      load(await branchesApi.computeSettings(orgId, pid, bid))
    } catch (e) {
      toast.error(getAppError(e))
    } finally {
      loading.value = false
    }
  },
  { immediate: true },
)

const steps = computed(() => {
  const max = settings.value?.max_shared_buffers_mb ?? 128
  const values = SHARED_BUFFERS_STEPS.filter((v) => v <= max)
  if (!values.includes(sharedBuffersMb.value)) {
    values.push(sharedBuffersMb.value)
    values.sort((a, b) => a - b)
  }
  return values
})

const sharedBuffersIndex = computed({
  get: () => [Math.max(steps.value.indexOf(sharedBuffersMb.value), 0)],
  set: (v: number[]) => { sharedBuffersMb.value = steps.value[v[0] ?? 0] ?? sharedBuffersMb.value },
})

const derived = computed(() => derive(sharedBuffersMb.value))

const isDirty = computed(() => {
  const s = settings.value
  if (!s) return false
  if (s.shared_buffers_mb !== sharedBuffersMb.value) return true
  return OVERRIDE_FIELDS.some((f) => {
    const saved = s.overrides[f.key]
    return (saved === null ? '' : String(saved)) !== String(overrides.value[f.key]).trim()
  })
})

const memory = computed(() => settings.value?.memory ?? null)
const projectedTotalMb = computed(() => (memory.value?.other_running_shared_buffers_mb ?? 0) + sharedBuffersMb.value)
const budgetUsagePercent = computed(() => {
  const budget = memory.value?.shared_buffers_budget_mb ?? 0
  return budget > 0 ? Math.min((projectedTotalMb.value / budget) * 100, 100) : 0
})
const overBudget = computed(() => !!memory.value && projectedTotalMb.value > memory.value.shared_buffers_budget_mb)
const recommendedIsApplied = computed(() => memory.value?.recommended_shared_buffers_mb === sharedBuffersMb.value)

function applyRecommended() {
  if (memory.value) sharedBuffersMb.value = memory.value.recommended_shared_buffers_mb
}

function parseOverride(value: string | number): number | null {
  const trimmed = String(value).trim()
  return trimmed ? Number(trimmed) : null
}

function onSubmit() {
  if (branch.value?.endpoint_status === 'running') {
    restartConfirmOpen.value = true
  } else {
    save()
  }
}

async function save() {
  const orgId = organizationStore.selectedOrganizationId
  if (!orgId || !projectId.value || !branchId.value) return
  submitting.value = true
  try {
    const updated = await branchesApi.updateComputeSettings(orgId, projectId.value, branchId.value, {
      shared_buffers_mb: sharedBuffersMb.value,
      effective_cache_size_mb: parseOverride(overrides.value.effective_cache_size_mb),
      maintenance_work_mem_mb: parseOverride(overrides.value.maintenance_work_mem_mb),
      work_mem_mb: parseOverride(overrides.value.work_mem_mb),
      max_connections: parseOverride(overrides.value.max_connections),
    })
    load(updated)
    restartConfirmOpen.value = false
    toast.success('Compute settings saved')
    await branchStore.fetch(orgId, projectId.value, true)
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div v-if="loading || !settings" class="flex justify-center py-12">
    <Loader2 class="size-6 animate-spin" />
  </div>

  <div v-else class="w-full divide-y">
    <section class="flex flex-col gap-4 py-8 md:grid md:grid-cols-[280px_1fr] md:gap-8">
      <div>
        <h3 class="text-sm font-medium">Server memory</h3>
        <p class="mt-1 text-sm text-muted-foreground">
          The shared_buffers of all running computes should together stay within about 25% of the host's RAM.
        </p>
      </div>
      <div v-if="memory" class="space-y-4">
        <dl class="grid grid-cols-2 gap-x-6 gap-y-3 text-sm sm:grid-cols-3">
          <div>
            <dt class="text-xs text-muted-foreground">Host RAM</dt>
            <dd class="font-medium">{{ formatMb(memory.host_total_mb) }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">Available now</dt>
            <dd class="font-medium">{{ formatMb(memory.host_available_mb) }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">shared_buffers budget</dt>
            <dd class="font-medium">{{ formatMb(memory.shared_buffers_budget_mb) }}</dd>
          </div>
          <div class="col-span-2 sm:col-span-3">
            <dt class="text-xs text-muted-foreground">Other running computes</dt>
            <dd class="font-medium">
              {{ memory.other_running_computes }}
              <span class="font-normal text-muted-foreground">
                using {{ formatMb(memory.other_running_shared_buffers_mb) }} of shared_buffers in total
              </span>
            </dd>
          </div>
        </dl>

        <div class="space-y-1.5">
          <div class="flex justify-between text-xs text-muted-foreground">
            <span>Others + this branch: {{ formatMb(projectedTotalMb) }}</span>
            <span>Budget: {{ formatMb(memory.shared_buffers_budget_mb) }}</span>
          </div>
          <Progress :model-value="budgetUsagePercent" :indicator-class="overBudget ? 'bg-destructive' : 'bg-primary'" />
        </div>

        <Alert v-if="overBudget" variant="destructive">
          <TriangleAlert />
          <AlertTitle>Over the memory budget</AlertTitle>
          <AlertDescription>
            Running this branch with {{ formatMb(sharedBuffersMb) }} would bring all computes to
            {{ formatMb(projectedTotalMb) }} of shared_buffers. That leaves too little RAM for work_mem, connections and the OS page cache.
          </AlertDescription>
        </Alert>

        <Alert>
          <Lightbulb />
          <AlertTitle>Recommended: {{ formatMb(memory.recommended_shared_buffers_mb) }}</AlertTitle>
          <AlertDescription>
            <p>This is the largest step that fits in the budget remaining after the other running computes.</p>
            <Button
              :disabled="recommendedIsApplied"
              size="sm"
              variant="outline"
              class="mt-2 cursor-pointer"
              @click="applyRecommended"
            >
              Use {{ formatMb(memory.recommended_shared_buffers_mb) }}
            </Button>
          </AlertDescription>
        </Alert>
      </div>
    </section>

    <section class="flex flex-col gap-4 py-8 md:grid md:grid-cols-[280px_1fr] md:gap-8">
      <div>
        <h3 class="text-sm font-medium">Compute memory</h3>
        <p class="mt-1 text-sm text-muted-foreground">
          shared_buffers sets the scale of the compute, and the other parameters follow it automatically.
          Type a value to override a parameter. Saving restarts a running compute.
        </p>
      </div>
      <form class="space-y-6" @submit.prevent="onSubmit">
        <div class="grid gap-3">
          <div class="flex items-baseline justify-between">
            <Label>shared_buffers</Label>
            <span class="text-sm font-medium tabular-nums">{{ formatMb(sharedBuffersMb) }}</span>
          </div>
          <Slider :min="0" :max="steps.length - 1" :step="1" v-model="sharedBuffersIndex" />
          <div class="flex justify-between text-xs text-muted-foreground">
            <span>{{ formatMb(steps[0] ?? 64) }}</span>
            <span>{{ formatMb(steps[steps.length - 1] ?? 64) }}</span>
          </div>
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div v-for="field in OVERRIDE_FIELDS" :key="field.key" class="grid gap-2">
            <div class="flex h-6 items-center gap-2">
              <Label :for="`override-${field.key}`" class="font-mono text-xs">{{ field.label }}</Label>
              <Badge v-if="!String(overrides[field.key]).trim()" variant="secondary">auto</Badge>
              <button
                v-else
                type="button"
                class="ml-auto inline-flex cursor-pointer items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
                @click="overrides[field.key] = ''"
              >
                <RotateCcw class="size-3" />
                Reset
              </button>
            </div>
            <div class="flex items-center gap-2">
              <Input
                :id="`override-${field.key}`"
                v-model="overrides[field.key]"
                type="number"
                :min="1"
                :placeholder="String(derived[field.key])"
              />
              <span v-if="field.unit" class="shrink-0 text-sm text-muted-foreground">{{ field.unit }}</span>
            </div>
            <p class="text-xs text-muted-foreground">{{ field.hint }}</p>
          </div>
        </div>

        <Button type="submit" class="cursor-pointer" :disabled="submitting || !isDirty">
          <Loader2 v-if="submitting" class="mr-1.5 size-3.5 animate-spin" />
          Save
        </Button>
      </form>
    </section>

    <AlertDialog v-model:open="restartConfirmOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Restart compute?</AlertDialogTitle>
          <AlertDialogDescription>
            The compute of this branch is running. Saving restarts it, and active connections will be dropped.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="submitting">Cancel</AlertDialogCancel>
          <AlertDialogAction class="cursor-pointer" :disabled="submitting" @click.prevent="save">
            <Loader2 v-if="submitting" class="mr-1.5 size-3.5 animate-spin" />
            Save and restart
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
