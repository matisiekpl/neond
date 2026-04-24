<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import VChart from 'vue-echarts'
import { connect, use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import { DataZoomInsideComponent, GridComponent, ToolboxComponent, TooltipComponent } from 'echarts/components'
import { SVGRenderer } from 'echarts/renderers'
import type { EChartsOption } from 'echarts'
import { CHART_COLORS, formatMetricValue } from '@/lib/metricPresets'
import { useMetricStore } from '@/stores/metric.store'
import type { MetricChartDefinition } from '@/types/dto/metricChartDefinition'

use([LineChart, GridComponent, TooltipComponent, DataZoomInsideComponent, ToolboxComponent, SVGRenderer])

const CHART_GROUP = 'metrics'
connect(CHART_GROUP)

const props = defineProps<{ chart: MetricChartDefinition }>()
const metricStore = useMetricStore()

const GAP_MS = 20_000

const colors = computed(() =>
  props.chart.series.map(
    (series, index) => series.color ?? CHART_COLORS[index % CHART_COLORS.length],
  ),
)

const chartRef = ref<InstanceType<typeof VChart> | null>(null)

async function enableRangeSelect(): Promise<void> {
  await nextTick()
  chartRef.value?.dispatchAction({
    type: 'takeGlobalCursor',
    key: 'dataZoomSelect',
    dataZoomSelectActive: true,
  })
}

type ZoomBatch = { startValue?: number; endValue?: number; start?: number; end?: number }
type ZoomEvent = { batch?: ZoomBatch[]; startValue?: number; endValue?: number }

function onDataZoom(event: ZoomEvent): void {
  const batch = event.batch?.[0] ?? event
  const start = batch.startValue
  const end = batch.endValue
  if (typeof start !== 'number' || typeof end !== 'number') return
  if (start === metricStore.rangeStart && end === metricStore.rangeEnd) return
  metricStore.setCustomRange(new Date(start), new Date(end))
}

const option = computed<EChartsOption>(() => {
  const series = props.chart.series.map((entry, index) => {
    const points = metricStore.seriesBySlug.get(entry.slug) ?? []
    const data: [number, number | null][] = []
    for (let pointIndex = 0; pointIndex < points.length; pointIndex += 1) {
      const point = points[pointIndex]
      data.push([point.x, point.y])
      const next = points[pointIndex + 1]
      if (next && next.x - point.x > GAP_MS) {
        data.push([point.x + 1, null])
      }
    }
    return {
      name: entry.label,
      type: 'line' as const,
      showSymbol: false,
      smooth: false,
      lineStyle: { width: 1.5 },
      color: colors.value[index],
      data,
    }
  })

  return {
    animation: false,
    grid: { top: 8, bottom: 28, left: 72, right: 12 },
    toolbox: {
      show: true,
      itemSize: 0,
      showTitle: false,
      feature: {
        dataZoom: { yAxisIndex: 'none' },
      },
    },
    dataZoom: [
      {
        type: 'inside',
        xAxisIndex: 0,
        filterMode: 'none',
        zoomOnMouseWheel: false,
        moveOnMouseMove: false,
      },
    ],
    xAxis: {
      type: 'time',
      min: metricStore.rangeStart,
      max: metricStore.rangeEnd,
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { show: false },
      splitNumber: 6,
      axisLabel: {
        hideOverlap: true,
        formatter: (value: number) =>
          new Date(value).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
      },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { show: true, lineStyle: { color: 'rgba(0,0,0,0.06)' } },
      axisLabel: {
        formatter: (value: number) => formatMetricValue(value, props.chart.unit),
      },
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'line' },
      formatter: (params) => {
        const items = Array.isArray(params) ? params : [params]
        if (!items.length) return ''
        const firstValue = (items[0] as unknown as { value: [number, number | null] }).value
        const time = new Date(firstValue[0]).toLocaleString('en-US')
        const rows = items
          .map((item) => {
            const typed = item as unknown as { value: [number, number | null]; color: string; seriesName: string }
            const value = typed.value[1]
            if (value === null) return ''
            const formatted = formatMetricValue(value, props.chart.unit)
            return `<div style="display:flex;align-items:center;gap:8px;">
              <span style="width:8px;height:8px;border-radius:2px;background:${typed.color};"></span>
              <span style="flex:1;">${typed.seriesName}</span>
              <span style="font-family:var(--font-mono);font-weight:500;">${formatted}</span>
            </div>`
          })
          .filter(Boolean)
          .join('')
        return `<div style="display:flex;flex-direction:column;gap:4px;font-size:12px;">
          <div style="font-weight:600;">${time}</div>${rows}
        </div>`
      },
    },
    series,
  }
})
</script>

<template>
  <div class="flex flex-col gap-4 rounded-lg border p-4">
    <div class="flex items-start justify-between gap-4">
      <h3 class="text-sm font-semibold">{{ chart.title }}</h3>
      <div class="flex flex-wrap items-center justify-end gap-x-3 gap-y-1 text-xs text-muted-foreground">
        <div
          v-for="(entry, index) in chart.series"
          :key="entry.slug"
          class="flex items-center gap-1.5"
        >
          <span class="size-2 rounded-[2px]" :style="{ backgroundColor: colors[index] }" />
          {{ entry.label }}
        </div>
      </div>
    </div>
    <div class="relative h-56">
      <VChart
        ref="chartRef"
        :group="CHART_GROUP"
        :option="option"
        autoresize
        @datazoom="onDataZoom"
        @finished="enableRangeSelect"
      />
    </div>
  </div>
</template>
