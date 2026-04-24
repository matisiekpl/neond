import type { MetricUnit } from '@/types/dto/metricUnit'
import type { MetricSeriesDefinition } from '@/types/dto/metricSeriesDefinition'

export interface MetricChartDefinition {
  id: string
  title: string
  unit: MetricUnit
  series: MetricSeriesDefinition[]
}
