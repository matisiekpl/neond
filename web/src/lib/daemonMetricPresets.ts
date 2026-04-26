import type { MetricChartDefinition } from '@/types/dto/metricChartDefinition'

export const daemonMetricCharts: MetricChartDefinition[] = [
  {
    id: 'pageserver-page-cache',
    title: 'Page cache',
    unit: 'count',
    series: [
      { slug: 'pageserver.page_cache.hits_total', label: 'Hits' },
      { slug: 'pageserver.page_cache.accesses_total', label: 'Accesses' },
    ],
  },
  {
    id: 'pageserver-page-cache-size',
    title: 'Page cache size',
    unit: 'bytes',
    series: [
      { slug: 'pageserver.page_cache.size_bytes', label: 'Size' },
    ],
  },
  {
    id: 'pageserver-tenants',
    title: 'Pageserver tenants',
    unit: 'count',
    series: [
      { slug: 'pageserver.tenant_states_count', label: 'Total' },
      { slug: 'pageserver.broken_tenants_count', label: 'Broken' },
    ],
  },
  {
    id: 'pageserver-io',
    title: 'Pageserver I/O',
    unit: 'bytes',
    series: [
      { slug: 'pageserver.io_operations_bytes_total', label: 'I/O bytes' },
    ],
  },
  {
    id: 'safekeeper-wal-throughput',
    title: 'WAL throughput',
    unit: 'bytes',
    series: [
      { slug: 'safekeeper.write_wal_bytes', label: 'Written bytes' },
      { slug: 'safekeeper.partial_backup_uploaded_bytes_total', label: 'Uploaded bytes' },
    ],
  },
  {
    id: 'safekeeper-wal-connections',
    title: 'WAL connections',
    unit: 'count',
    series: [
      { slug: 'safekeeper.wal_receivers', label: 'Receivers' },
      { slug: 'safekeeper.wal_readers', label: 'Readers' },
    ],
  },
  {
    id: 'safekeeper-segments',
    title: 'WAL segments',
    unit: 'count',
    series: [
      { slug: 'safekeeper.removed_wal_segments_total', label: 'Removed' },
      { slug: 'safekeeper.backed_up_segments_total', label: 'Backed up' },
    ],
  },
  {
    id: 'safekeeper-errors',
    title: 'Safekeeper errors',
    unit: 'count',
    series: [
      { slug: 'safekeeper.backup_errors_total', label: 'Backup errors' },
      { slug: 'safekeeper.evicted_timelines', label: 'Evicted timelines' },
    ],
  },
]