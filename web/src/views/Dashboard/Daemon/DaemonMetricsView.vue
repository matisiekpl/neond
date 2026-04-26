<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useTitle } from '@vueuse/core'
import MetricsDashboard from '@/elements/MetricsDashboard.vue'
import TimeWindowPicker from '@/elements/TimeWindowPicker.vue'
import { useOrganizationStore } from '@/stores/organization.store'
import { useMetricStore } from '@/stores/metric.store'
import { daemonMetricCharts } from '@/lib/daemonMetricPresets'

useTitle('Daemon monitoring — neond')

const organizationStore = useOrganizationStore()
const metricStore = useMetricStore()

const organizationId = computed(() => organizationStore.selectedOrganizationId)

onMounted(() => {
  if (organizationId.value) {
    metricStore.startDaemonPolling(organizationId.value)
  }
})

onUnmounted(() => {
  metricStore.reset()
})
</script>

<template>
  <div v-if="organizationId" class="flex h-full flex-col gap-4">
    <div class="flex items-center justify-end">
      <TimeWindowPicker />
    </div>
    <MetricsDashboard :charts="daemonMetricCharts" />
  </div>
</template>