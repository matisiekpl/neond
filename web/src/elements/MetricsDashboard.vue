<script setup lang="ts">
import { computed } from 'vue'
import { onKeyStroke } from '@vueuse/core'
import { InfoIcon } from 'lucide-vue-next'
import MetricChart from '@/elements/MetricChart.vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { useMetricStore, RANGE_PRESETS } from '@/stores/metric.store'
import type { MetricChartDefinition } from '@/types/dto/metricChartDefinition'

defineProps<{ charts: MetricChartDefinition[] }>()

const metricStore = useMetricStore()

const customFromLabel = computed(() => new Date(metricStore.rangeStart).toLocaleString('en-US'))
const customToLabel = computed(() => new Date(metricStore.rangeEnd).toLocaleString('en-US'))

function resetToDefault() {
  metricStore.setRange('30m')
}

function isTypingTarget(target: EventTarget | null): boolean {
  const element = target as HTMLElement | null
  return element?.tagName === 'INPUT' || element?.tagName === 'TEXTAREA'
}

RANGE_PRESETS.forEach((preset, index) => {
  onKeyStroke(String(index + 1), (event) => {
    if (isTypingTarget(event.target)) return
    event.preventDefault()
    metricStore.setRange(preset.value)
  })
})

onKeyStroke(['r', 'R'], (event) => {
  if (isTypingTarget(event.target)) return
  if (metricStore.isLive) return
  event.preventDefault()
  resetToDefault()
})
</script>

<template>
  <div class="flex h-full flex-col gap-4 overflow-auto">
    <Alert v-if="!metricStore.isLive" class="flex items-center justify-between gap-4">
      <InfoIcon />
      <AlertDescription class="flex flex-1 flex-wrap items-center gap-2">
        <span>Showing metrics for custom period</span>
        <code class="rounded bg-muted px-1.5 py-0.5 text-xs">{{ customFromLabel }}</code>
        <span>to</span>
        <code class="rounded bg-muted px-1.5 py-0.5 text-xs">{{ customToLabel }}</code>
      </AlertDescription>
      <Button variant="outline" size="sm" class="cursor-pointer gap-2" @click="resetToDefault">
        Reset
        <kbd class="flex size-4 items-center justify-center rounded border border-border bg-muted font-mono text-[10px] text-muted-foreground">R</kbd>
      </Button>
    </Alert>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <MetricChart v-for="chart in charts" :key="chart.id" :chart="chart" />
    </div>
  </div>
</template>