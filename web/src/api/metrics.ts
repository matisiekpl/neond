import api from '@/api/base'
import type { MetricSample } from '@/types/dto/metricSample'

function toNaiveUtc(date: Date): string {
  return date.toISOString().replace(/Z$/, '')
}

export const metricsApi = {
  async listForBranch(
    organizationId: string,
    projectId: string,
    branchId: string,
    from: Date,
    to: Date,
  ): Promise<MetricSample[]> {
    const response = await api.get<MetricSample[]>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}/endpoint/metrics`,
      { params: { from: toNaiveUtc(from), to: toNaiveUtc(to) } },
    )
    return response.data
  },

  async listDaemon(organizationId: string, from: Date, to: Date): Promise<MetricSample[]> {
    const response = await api.get<MetricSample[]>(
      `organizations/${organizationId}/daemon/metrics`,
      { params: { from: toNaiveUtc(from), to: toNaiveUtc(to) } },
    )
    return response.data
  },
}
