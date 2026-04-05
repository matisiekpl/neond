import { create } from "zustand"
import { toast } from "sonner"
import { projectsApi } from "~/api/projects"
import { branchesApi } from "~/api/branches"
import { getAppError } from "~/lib/errors"
import type { Project } from "~/types/models/project"
import type { CreateProjectRequest, UpdateProjectRequest } from "~/types/dto/project"

type ProjectState = {
  projects: Project[]
  loading: boolean
  reset: () => void
  fetchProjects: (organizationId: string, skipLoading?: boolean) => Promise<void>
  createProject: (organizationId: string, payload: CreateProjectRequest) => Promise<Project>
  updateProject: (organizationId: string, projectId: string, payload: UpdateProjectRequest) => Promise<void>
  deleteProject: (organizationId: string, projectId: string) => Promise<void>
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  projects: [],
  loading: false,

  reset: () => set({ projects: [], loading: false }),

  fetchProjects: async (organizationId, skipLoading = false) => {
    if (!skipLoading) set({ loading: true })
    try {
      const list = await projectsApi.list(organizationId)
      set({ projects: list })
    } finally {
      if (!skipLoading) set({ loading: false })
    }
  },

  createProject: async (organizationId, payload:CreateProjectRequest) => {
    try {
      const project = await projectsApi.create(organizationId, payload)
      await branchesApi.create(organizationId, project.id, "production")
      await get().fetchProjects(organizationId)
      toast.success("Project created")
      return project
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  updateProject: async (organizationId, projectId, payload) => {
    try {
      await projectsApi.update(organizationId, projectId, payload)
      await get().fetchProjects(organizationId, true)
      toast.success("Project updated")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  deleteProject: async (organizationId, projectId) => {
    try {
      await projectsApi.remove(organizationId, projectId)
      await get().fetchProjects(organizationId)
      toast.success("Project deleted")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },
}))
