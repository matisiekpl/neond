<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useTitle } from '@vueuse/core'
import MetricsDashboard from '@/elements/MetricsDashboard.vue'
import TimeWindowPicker from '@/elements/TimeWindowPicker.vue'
import { useMetricStore } from '@/stores/metric.store'
import { daemonMetricCharts } from '@/lib/daemonMetricPresets'

useTitle('Daemon monitoring — neond')

const metricStore = useMetricStore()

onMounted(() => {
  metricStore.startDaemonPolling()
})

onUnmounted(() => {
  metricStore.reset()
})
</script>

<template>
  <div class="flex h-full flex-col gap-4">
    <div class="flex items-center justify-end">
      <TimeWindowPicker />
    </div>
    <MetricsDashboard :charts="daemonMetricCharts" />
  </div>
</template>