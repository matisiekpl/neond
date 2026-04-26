import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { toast } from 'vue-sonner'
import { useIntervalFn } from '@vueuse/core'
import { metricsApi } from '@/api/metrics'
import { getAppError } from '@/api/utils'
import type { MetricSample } from '@/types/dto/metricSample'

export type MetricRange = '5m' | '15m' | '30m' | '1h' | '3h' | '6h' | '12h' | '24h'
export type MetricMode = 'branch' | 'daemon'

const CPU_PERCENT = 'cpu.percent'

export const RANGE_PRESETS: { value: MetricRange; label: string }[] = [
  { value: '5m', label: 'Last 5 minutes' },
  { value: '15m', label: 'Last 15 minutes' },
  { value: '30m', label: 'Last 30 minutes' },
  { value: '1h', label: 'Last hour' },
  { value: '3h', label: 'Last 3 hours' },
  { value: '6h', label: 'Last 6 hours' },
  { value: '12h', label: 'Last 12 hours' },
  { value: '24h', label: 'Last 24 hours' },
]

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
  const mode = ref<MetricMode>('branch')
  const branchScope = ref<{ organizationId: string; projectId: string; branchId: string } | null>(null)
  const daemonScope = ref<{ organizationId: string } | null>(null)

  const { pause, resume } = useIntervalFn(() => fetch(true), COLLECTION_INTERVAL_MS, { immediate: false })

  const canFetch = computed(() =>
    mode.value === 'branch' ? branchScope.value !== null : daemonScope.value !== null,
  )

  async function fetch(silent = false): Promise<void> {
    if (!canFetch.value) return
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
      if (mode.value === 'branch' && branchScope.value) {
        const { organizationId, projectId, branchId } = branchScope.value
        samples.value = await metricsApi.listForBranch(organizationId, projectId, branchId, from, to)
      } else if (mode.value === 'daemon' && daemonScope.value) {
        samples.value = await metricsApi.listDaemon(daemonScope.value.organizationId, from, to)
      }
    } catch (error) {
      toast.error(getAppError(error))
    } finally {
      if (!silent) loading.value = false
    }
  }

  function startBranchPolling(organizationId: string, projectId: string, branchId: string): void {
    stopPolling()
    mode.value = 'branch'
    branchScope.value = { organizationId, projectId, branchId }
    fetch()
    if (range.value !== null) resume()
  }

  function startDaemonPolling(organizationId: string): void {
    stopPolling()
    mode.value = 'daemon'
    daemonScope.value = { organizationId }
    fetch()
    if (range.value !== null) resume()
  }

  function stopPolling(): void {
    pause()
    branchScope.value = null
    daemonScope.value = null
  }

  function setRange(next: MetricRange): void {
    range.value = next
    resume()
    fetch()
  }

  function setCustomRange(from: Date, to: Date): void {
    range.value = null
    rangeStart.value = from.getTime()
    rangeEnd.value = to.getTime()
    pause()
    fetch()
  }

  const isLive = computed<boolean>(() => range.value !== null)

  function refresh(): void {
    fetch()
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
    const downtimeSamples = mode.value === 'branch'
      ? samples.value.filter((sample) => sample.slug === CPU_PERCENT)
      : samples.value
    const timestamps = Array.from(
      new Set(downtimeSamples.map((sample) => new Date(sample.recorded_at + 'Z').getTime())),
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

  const isFullyDown = computed<boolean>(() => {
    if (loading.value) return false
    const sorted = [...downtimeRanges.value].sort((a, b) => a.start - b.start)
    let covered = rangeStart.value
    for (const range of sorted) {
      if (range.start > covered) break
      covered = Math.max(covered, range.end)
      if (covered >= rangeEnd.value) return true
    }
    return false
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
    mode,
    isLive,
    fetch,
    refresh,
    startBranchPolling,
    startDaemonPolling,
    stopPolling,
    setRange,
    setCustomRange,
    seriesBySlug,
    downtimeRanges,
    isFullyDown,
    reset,
  }
})