import type { Branch } from "~/types/models/branch"
import api from "./client"

export const branchesApi = {
  async list(orgId: string, projectId: string): Promise<Branch[]> {
    const response = await api.get<Branch[]>(
      `organizations/${orgId}/projects/${projectId}/branches`,
    )
    return response.data
  },

  async create(orgId: string, projectId: string, name: string, parentBranchId?: string): Promise<Branch> {
    const response = await api.post<Branch>(
      `organizations/${orgId}/projects/${projectId}/branches`,
      { name, ...(parentBranchId ? { parent_branch_id: parentBranchId } : {}) },
    )
    return response.data
  },

  async rename(orgId: string, projectId: string, branchId: string, name: string): Promise<Branch> {
    const response = await api.put<Branch>(
      `organizations/${orgId}/projects/${projectId}/branches/${branchId}`,
      { name },
    )
    return response.data
  },

  async launch(orgId: string, projectId: string, branchId: string): Promise<void> {
    await api.post(`organizations/${orgId}/projects/${projectId}/branches/${branchId}/endpoint`)
  },

  async shutdown(orgId: string, projectId: string, branchId: string): Promise<void> {
    await api.delete(`organizations/${orgId}/projects/${projectId}/branches/${branchId}/endpoint`)
  },

  async remove(orgId: string, projectId: string, branchId: string): Promise<void> {
    await api.delete(`organizations/${orgId}/projects/${projectId}/branches/${branchId}`)
  },
}
