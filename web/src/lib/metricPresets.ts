import type { MetricUnit } from '@/types/dto/metricUnit'
import type { MetricChartDefinition } from '@/types/dto/metricChartDefinition'
import { formatBytes } from '@/lib/utils'

export const CHART_COLORS = [
  '#f97316',
  '#06b6d4',
  '#10b981',
  '#8b5cf6',
  '#ec4899',
]

export const metricCharts: MetricChartDefinition[] = [
  {
    id: 'cpu',
    title: 'CPU',
    unit: 'percent',
    series: [{ slug: 'cpu.percent', label: 'CPU %' }],
  },
  {
    id: 'memory',
    title: 'Memory',
    unit: 'bytes',
    series: [{ slug: 'mem.rss', label: 'RSS' }],
  },
  {
    id: 'pg-connections',
    title: 'Postgres connections',
    unit: 'count',
    series: [
      { slug: 'pg.connections.total', label: 'Total' },
      { slug: 'pg.connections.active', label: 'Active' },
      { slug: 'pg.connections.idle', label: 'Idle' },
    ],
  },
  {
    id: 'compute-ctl',
    title: 'compute_ctl',
    unit: 'raw',
    series: [
      { slug: 'compute_ctl.up', label: 'Up' },
      { slug: 'compute_ctl.pg_downtime_ms', label: 'PG downtime (ms)' },
      { slug: 'compute_ctl.pagestream_errors_total', label: 'Pagestream errors' },
    ],
  },
]

export function formatMetricValue(value: number, unit: MetricUnit): string {
  switch (unit) {
    case 'percent':
      return `${value.toFixed(1)}%`
    case 'bytes':
      return formatBytes(value)
    case 'milliseconds':
      return `${value.toFixed(0)} ms`
    case 'count':
    case 'raw':
    default:
      return value.toLocaleString()
  }
}
