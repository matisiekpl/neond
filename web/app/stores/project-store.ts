import { create } from "zustand"
import { toast } from "sonner"
import { projectsApi } from "~/api/projects"
import { getAppError } from "~/lib/errors"
import type { Project } from "~/types/models/project"
import type { CreateProjectRequest, UpdateProjectRequest } from "~/types/dto/project"

type ProjectState = {
  projects: Project[]
  loading: boolean
  reset: () => void
  fetchProjects: (orgId: string) => Promise<void>
  createProject: (orgId: string, dto: CreateProjectRequest) => Promise<Project>
  updateProject: (orgId: string, projectId: string, dto: UpdateProjectRequest) => Promise<void>
  deleteProject: (orgId: string, projectId: string) => Promise<void>
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  projects: [],
  loading: false,

  reset: () => set({ projects: [], loading: false }),

  fetchProjects: async (orgId) => {
    set({ loading: true })
    try {
      const list = await projectsApi.list(orgId)
      set({ projects: list })
    } finally {
      set({ loading: false })
    }
  },

  createProject: async (orgId, dto) => {
    try {
      const project = await projectsApi.create(orgId, dto)
      await get().fetchProjects(orgId)
      toast.success("Project created")
      return project
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  updateProject: async (orgId, projectId, dto) => {
    try {
      await projectsApi.update(orgId, projectId, dto)
      await get().fetchProjects(orgId)
      toast.success("Project updated")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  deleteProject: async (orgId, projectId) => {
    try {
      await projectsApi.remove(orgId, projectId)
      await get().fetchProjects(orgId)
      toast.success("Project deleted")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },
}))
