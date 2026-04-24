import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { toast } from 'vue-sonner'
import { metricsApi } from '@/api/metrics'
import { getAppError } from '@/api/utils'
import type { MetricSample } from '@/types/dto/metricSample'

export type MetricRange = '5m' | '15m' | '30m' | '1h' | '3h' | '6h' | '12h' | '24h'

export const rangeDurationMs: Record<MetricRange, number> = {
  '5m': 5 * 60 * 1000,
  '15m': 15 * 60 * 1000,
  '30m': 30 * 60 * 1000,
  '1h': 60 * 60 * 1000,
  '3h': 3 * 60 * 60 * 1000,
  '6h': 6 * 60 * 60 * 1000,
  '12h': 12 * 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
}

const COLLECTION_INTERVAL_MS = 10_000
const DOWNTIME_THRESHOLD_MS = 2 * COLLECTION_INTERVAL_MS

export type DowntimeRange = { start: number; end: number }

export type MetricSeriesPoint = { x: number; y: number }

export const useMetricStore = defineStore('metric', () => {
  const samples = ref<MetricSample[]>([])
  const loading = ref(false)
  const range = ref<MetricRange | null>('30m')
  const rangeStart = ref<number>(Date.now() - rangeDurationMs['30m'])
  const rangeEnd = ref<number>(Date.now())
  let pollTimer: ReturnType<typeof setInterval> | null = null
  let currentScope: { organizationId: string; projectId: string; branchId: string } | null = null

  async function fetch(organizationId: string, projectId: string, branchId: string, silent = false): Promise<void> {
    if (!silent) loading.value = true
    try {
      let from: Date
      let to: Date
      if (range.value !== null) {
        to = new Date()
        from = new Date(to.getTime() - rangeDurationMs[range.value])
        rangeStart.value = from.getTime()
        rangeEnd.value = to.getTime()
      } else {
        from = new Date(rangeStart.value)
        to = new Date(rangeEnd.value)
      }
      samples.value = await metricsApi.list(organizationId, projectId, branchId, from, to)
    } catch (error) {
      toast.error(getAppError(error))
    } finally {
      if (!silent) loading.value = false
    }
  }

  function startInterval(): void {
    if (pollTimer || !currentScope) return
    pollTimer = setInterval(() => {
      if (!currentScope) return
      fetch(currentScope.organizationId, currentScope.projectId, currentScope.branchId, true)
    }, COLLECTION_INTERVAL_MS)
  }

  function stopInterval(): void {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  function startPolling(organizationId: string, projectId: string, branchId: string): void {
    stopPolling()
    currentScope = { organizationId, projectId, branchId }
    fetch(organizationId, projectId, branchId)
    if (range.value !== null) {
      startInterval()
    }
  }

  function stopPolling(): void {
    stopInterval()
    currentScope = null
  }

  function setRange(next: MetricRange): void {
    range.value = next
    startInterval()
    if (currentScope) {
      fetch(currentScope.organizationId, currentScope.projectId, currentScope.branchId)
    }
  }

  function setCustomRange(from: Date, to: Date): void {
    range.value = null
    rangeStart.value = from.getTime()
    rangeEnd.value = to.getTime()
    stopInterval()
    if (currentScope) {
      fetch(currentScope.organizationId, currentScope.projectId, currentScope.branchId)
    }
  }

  const isLive = computed<boolean>(() => range.value !== null)

  function refresh(): void {
    if (currentScope) {
      fetch(currentScope.organizationId, currentScope.projectId, currentScope.branchId)
    }
  }

  const seriesBySlug = computed<Map<string, MetricSeriesPoint[]>>(() => {
    const map = new Map<string, MetricSeriesPoint[]>()
    for (const sample of samples.value) {
      const arr = map.get(sample.slug) ?? []
      arr.push({ x: new Date(sample.recorded_at + 'Z').getTime(), y: sample.value })
      map.set(sample.slug, arr)
    }
    for (const arr of map.values()) {
      arr.sort((a, b) => a.x - b.x)
    }
    return map
  })

  const downtimeRanges = computed<DowntimeRange[]>(() => {
    const start = rangeStart.value
    const end = rangeEnd.value
    const timestamps = Array.from(
      new Set(samples.value.map((sample) => new Date(sample.recorded_at + 'Z').getTime())),
    ).sort((a, b) => a - b)

    if (timestamps.length === 0) {
      return [{ start, end }]
    }

    const ranges: DowntimeRange[] = []
    if (timestamps[0] - start > DOWNTIME_THRESHOLD_MS) {
      ranges.push({ start, end: timestamps[0] })
    }
    for (let index = 1; index < timestamps.length; index += 1) {
      const previous = timestamps[index - 1]
      const current = timestamps[index]
      if (current - previous > DOWNTIME_THRESHOLD_MS) {
        ranges.push({ start: previous, end: current })
      }
    }
    const last = timestamps[timestamps.length - 1]
    if (end - last > DOWNTIME_THRESHOLD_MS) {
      ranges.push({ start: last, end })
    }
    return ranges
  })

  function reset(): void {
    stopPolling()
    samples.value = []
    loading.value = false
    range.value = '30m'
  }

  return {
    samples,
    loading,
    range,
    rangeStart,
    rangeEnd,
    isLive,
    fetch,
    refresh,
    startPolling,
    stopPolling,
    setRange,
    setCustomRange,
    seriesBySlug,
    downtimeRanges,
    reset,
  }
})
