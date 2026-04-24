<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { InfoIcon } from 'lucide-vue-next'
import MetricChart from '@/elements/MetricChart.vue'
import TimeWindowPicker from '@/elements/TimeWindowPicker.vue'
import { useOrganizationStore } from '@/stores/organization.store'
import { useMetricStore } from '@/stores/metric.store'
import { metricCharts } from '@/lib/metricPresets'

const route = useRoute()
const organizationStore = useOrganizationStore()
const metricStore = useMetricStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)
const organizationId = computed(() => organizationStore.selectedOrganizationId)

const customFromLabel = computed(() => new Date(metricStore.rangeStart).toLocaleString())
const customToLabel = computed(() => new Date(metricStore.rangeEnd).toLocaleString())

function resetToDefault() {
  metricStore.setRange('30m')
}

onMounted(() => {
  if (organizationId.value) {
    metricStore.startPolling(organizationId.value, projectId.value, branchId.value)
  }
})

onUnmounted(() => {
  metricStore.reset()
})

watch(
  () => [organizationId.value, projectId.value, branchId.value] as const,
  ([nextOrganizationId, nextProjectId, nextBranchId]) => {
    if (nextOrganizationId) {
      metricStore.startPolling(nextOrganizationId, nextProjectId, nextBranchId)
    }
  },
)
</script>

<template>
  <div v-if="organizationId" class="flex h-full flex-col gap-4 overflow-auto">
    <div class="flex items-center justify-end gap-2">
      <TimeWindowPicker />
    </div>

    <div
      v-if="!metricStore.isLive"
      class="flex items-center justify-between gap-4 rounded-md border border-blue-200 bg-blue-50 px-4 py-2.5 text-sm text-blue-900 dark:border-blue-900/50 dark:bg-blue-950/30 dark:text-blue-200"
    >
      <div class="flex items-center gap-2">
        <InfoIcon class="size-4" />
        <span>Showing metrics for custom period</span>
        <code class="rounded bg-white/60 px-1.5 py-0.5 text-xs dark:bg-black/20">{{ customFromLabel }}</code>
        <span>to</span>
        <code class="rounded bg-white/60 px-1.5 py-0.5 text-xs dark:bg-black/20">{{ customToLabel }}</code>
      </div>
      <button
        class="cursor-pointer text-sm font-medium text-blue-700 hover:underline dark:text-blue-300"
        @click="resetToDefault"
      >
        Reset
      </button>
    </div>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <MetricChart v-for="chart in metricCharts" :key="chart.id" :chart="chart" />
    </div>
  </div>
</template>
