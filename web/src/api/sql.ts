import api from '@/api/base'
import type { ExecuteSqlResponse } from '@/types/dto/executeSqlResponse'

export const sqlApi = {
  async execute(
    organizationId: string,
    projectId: string,
    branchId: string,
    sql: string,
    lsn?: string | null,
  ): Promise<ExecuteSqlResponse> {
    const response = await api.post<ExecuteSqlResponse>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}/sql`,
      { sql, ...(lsn != null ? { lsn } : {}) },
    )
    return response.data
  },
}
