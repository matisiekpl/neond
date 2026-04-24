<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTitle } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { Check, Copy } from 'lucide-vue-next'
import { useProjectStore } from '@/stores/project.store'
import { useBranchStore } from '@/stores/branch.store'
import { useOrganizationStore } from '@/stores/organization.store'
import EndpointStatusBadge from '@/elements/EndpointStatusBadge.vue'
import TimeWindowPicker from '@/elements/TimeWindowPicker.vue'

const copied = ref(false)

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const branchStore = useBranchStore()
const organizationStore = useOrganizationStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)

const project = computed(() => projectStore.projects.find((p) => p.id === projectId.value))
const branch = computed(() => branchStore.branches.find((b) => b.id === branchId.value))

const isMetricsRoute = computed(() => route.name === 'projects.branches.metrics')

async function copyBranchId() {
  if (!branch.value) return
  await navigator.clipboard.writeText(branch.value.id)
  copied.value = true
  toast.success('Branch ID copied')
  setTimeout(() => { copied.value = false }, 1500)
}

useTitle(
  computed(() =>
    project.value && branch.value
      ? `${project.value.name} / ${branch.value.name} — neond`
      : 'neond',
  ),
)
</script>

<template>
  <div
    v-if="!branch"
    class="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center"
  >
    <p class="text-sm font-medium">Branch not found</p>
    <p class="mt-1 text-sm text-muted-foreground">
      This branch may have been deleted or you don't have access to it.
    </p>
    <button
      type="button"
      class="mt-4 text-sm underline underline-offset-4"
      @click="router.push({ name: 'projects.show', params: { organizationId: organizationStore.selectedOrganizationId, projectId } })"
    >
      Back to project
    </button>
  </div>

  <div v-else class="flex flex-col gap-6 h-full">
    <div class="shrink-0 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
      <div class="min-w-0">
        <div class="flex flex-wrap items-center gap-2">
          <h1 class="text-lg font-semibold truncate">{{ branch.name }}</h1>
          <EndpointStatusBadge :status="branch.endpoint_status" />
        </div>
        <div class="mt-1 flex max-w-full flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted-foreground">
          <button
            type="button"
            class="group inline-flex max-w-full cursor-pointer items-center gap-1.5 overflow-hidden rounded border border-border bg-muted/50 px-2 py-0.5 font-mono text-xs transition-colors hover:bg-muted"
            :title="`Copy branch ID: ${branch.id}`"
            @click="copyBranchId"
          >
            <span class="shrink-0 text-[10px] uppercase tracking-wider text-muted-foreground/70">ID</span>
            <span class="truncate">{{ branch.id }}</span>
            <Check v-if="copied" class="size-3 shrink-0 text-emerald-500" />
            <Copy v-else class="size-3 shrink-0 opacity-50 group-hover:opacity-100" />
          </button>
        </div>
      </div>
      <TimeWindowPicker v-if="isMetricsRoute" />
    </div>

    <div class="flex-1 min-h-0">
      <RouterView />
    </div>
  </div>
</template>