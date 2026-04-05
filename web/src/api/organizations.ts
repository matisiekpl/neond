import api from '@/api/base'
import type { Organization } from '@/types/models/organization'

export const organizationsApi = {
  async list(): Promise<Organization[]> {
    const response = await api.get<Organization[]>('organizations')
    return response.data
  },

  async create(name: string): Promise<Organization> {
    const response = await api.post<Organization>('organizations', { name })
    return response.data
  },

  async update(organizationId: string, name: string): Promise<Organization> {
    const response = await api.put<Organization>(`organizations/${organizationId}`, { name })
    return response.data
  },

  async remove(organizationId: string): Promise<void> {
    await api.delete(`organizations/${organizationId}`)
  },
}
