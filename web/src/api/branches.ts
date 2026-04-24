import api from '@/api/base'
import type { Branch } from '@/types/models/branch'
import type { LsnResponse } from '@/types/dto/lsn'

export const branchesApi = {
  async list(organizationId: string, projectId: string): Promise<Branch[]> {
    const response = await api.get<Branch[]>(`organizations/${organizationId}/projects/${projectId}/branches`)
    return response.data
  },

  async create(organizationId: string, projectId: string, name: string, parentBranchId?: string): Promise<Branch> {
    const response = await api.post<Branch>(
      `organizations/${organizationId}/projects/${projectId}/branches`,
      { name, ...(parentBranchId ? { parent_branch_id: parentBranchId } : {}) },
    )
    return response.data
  },

  async update(organizationId: string, projectId: string, branchId: string, name: string): Promise<Branch> {
    const response = await api.put<Branch>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}`,
      { name },
    )
    return response.data
  },

  async launch(organizationId: string, projectId: string, branchId: string): Promise<void> {
    await api.post(`organizations/${organizationId}/projects/${projectId}/branches/${branchId}/endpoint`)
  },

  async shutdown(organizationId: string, projectId: string, branchId: string): Promise<void> {
    await api.delete(`organizations/${organizationId}/projects/${projectId}/branches/${branchId}/endpoint`)
  },

  async remove(organizationId: string, projectId: string, branchId: string): Promise<void> {
    await api.delete(`organizations/${organizationId}/projects/${projectId}/branches/${branchId}`)
  },

  async lsn(organizationId: string, projectId: string, branchId: string, timestamp: Date): Promise<LsnResponse> {
    const response = await api.get<LsnResponse>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}/lsn`,
      { params: { timestamp: timestamp.toISOString() } },
    )
    return response.data
  },

  async changePassword(organizationId: string, projectId: string, branchId: string, password: string): Promise<Branch> {
    const response = await api.put<Branch>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}/password`,
      { password },
    )
    return response.data
  },

  async restore(organizationId: string, projectId: string, branchId: string, lsn: string): Promise<Branch> {
    const response = await api.post<Branch>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}/restore`,
      { lsn },
    )
    return response.data
  },

  async resetToParent(organizationId: string, projectId: string, branchId: string): Promise<Branch> {
    const response = await api.post<Branch>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}/reset`,
    )
    return response.data
  },

  async detach(organizationId: string, projectId: string, branchId: string): Promise<Branch> {
    const response = await api.post<Branch>(
      `organizations/${organizationId}/projects/${projectId}/branches/${branchId}/detach`,
    )
    return response.data
  },
}
