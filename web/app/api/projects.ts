import type { Project } from "~/types/models/project"
import type { CreateProjectRequest, UpdateProjectRequest } from "~/types/dto/project"
import api from "./client"

export const projectsApi = {
  async list(organizationId: string): Promise<Project[]> {
    const response = await api.get<Project[]>(`organizations/${organizationId}/projects`)
    return response.data
  },

  async get(organizationId: string, projectId: string): Promise<Project> {
    const response = await api.get<Project>(`organizations/${organizationId}/projects/${projectId}`)
    return response.data
  },

  async create(organizationId: string, payload: CreateProjectRequest): Promise<Project> {
    const response = await api.post<Project>(`organizations/${organizationId}/projects`, payload)
    return response.data
  },

  async update(organizationId: string, projectId: string, payload: UpdateProjectRequest): Promise<Project> {
    const response = await api.put<Project>(
      `organizations/${organizationId}/projects/${projectId}`,
      payload,
    )
    return response.data
  },

  async remove(organizationId: string, projectId: string): Promise<void> {
    await api.delete(`organizations/${organizationId}/projects/${projectId}`)
  },
}
