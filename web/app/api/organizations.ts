import type { Organization } from "~/types/models"
import api from "./client"

export const organizationsApi = {
  async list(): Promise<Organization[]> {
    const response = await api.get<Organization[]>("organizations")
    return response.data
  },

  async create(name: string): Promise<Organization> {
    const response = await api.post<Organization>("organizations", { name })
    return response.data
  },

  async update(orgId: string, name: string): Promise<Organization> {
    const response = await api.put<Organization>(`organizations/${orgId}`, {
      name,
    })
    return response.data
  },

  async remove(orgId: string): Promise<void> {
    await api.delete(`organizations/${orgId}`)
  },
}
