import type { OrganizationMemberUser } from "~/types/models"
import api from "./client"

export const membersApi = {
  async list(orgId: string): Promise<OrganizationMemberUser[]> {
    const response = await api.get<OrganizationMemberUser[]>(
      `organizations/${orgId}/members`,
    )
    return response.data
  },

  async addByEmail(orgId: string, email: string): Promise<void> {
    await api.post(`organizations/${orgId}/members`, { email })
  },

  async remove(orgId: string, userId: string): Promise<void> {
    await api.delete(`organizations/${orgId}/members/${userId}`)
  },
}
