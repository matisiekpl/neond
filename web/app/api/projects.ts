import type { Project } from "~/types/models/project"
import type { CreateProjectDto, UpdateProjectDto } from "~/types/dto/project"
import api from "./client"

export const projectsApi = {
  async list(orgId: string): Promise<Project[]> {
    const response = await api.get<Project[]>(`organizations/${orgId}/projects`)
    return response.data
  },

  async create(orgId: string, dto: CreateProjectDto): Promise<Project> {
    const response = await api.post<Project>(`organizations/${orgId}/projects`, dto)
    return response.data
  },

  async update(orgId: string, projectId: string, dto: UpdateProjectDto): Promise<Project> {
    const response = await api.put<Project>(
      `organizations/${orgId}/projects/${projectId}`,
      dto,
    )
    return response.data
  },

  async remove(orgId: string, projectId: string): Promise<void> {
    await api.delete(`organizations/${orgId}/projects/${projectId}`)
  },
}
