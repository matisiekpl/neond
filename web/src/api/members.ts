import api from '@/api/base'
import type { OrganizationMemberUser } from '@/types/models/organization'

export const membersApi = {
  async list(organizationId: string): Promise<OrganizationMemberUser[]> {
    const response = await api.get<OrganizationMemberUser[]>(`organizations/${organizationId}/members`)
    return response.data
  },

  async assignByEmail(organizationId: string, email: string): Promise<void> {
    await api.post(`organizations/${organizationId}/members`, { email })
  },

  async remove(organizationId: string, userId: string): Promise<void> {
    await api.delete(`organizations/${organizationId}/members/${userId}`)
  },
}
