import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { projectsApi } from '@/api/projects'
import { branchesApi } from '@/api/branches'
import { getAppError } from '@/api/utils'
import type { Project } from '@/types/models/project'
import type { CreateProjectRequest, UpdateProjectRequest } from '@/types/dto/project'

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([])
  const loading = ref(false)

  function reset(): void {
    projects.value = []
    loading.value = false
  }

  async function fetch(organizationId: string, silent = false): Promise<void> {
    if (!silent) loading.value = true
    try {
      projects.value = await projectsApi.list(organizationId)
    } finally {
      if (!silent) loading.value = false
    }
  }

  async function create(organizationId: string, payload: CreateProjectRequest): Promise<Project> {
    try {
      const project = await projectsApi.create(organizationId, payload)
      await branchesApi.create(organizationId, project.id, 'production')
      await fetch(organizationId)
      toast.success('Project created')
      return project
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function update(organizationId: string, projectId: string, payload: UpdateProjectRequest): Promise<void> {
    try {
      await projectsApi.update(organizationId, projectId, payload)
      await fetch(organizationId, true)
      toast.success('Project updated')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function remove(organizationId: string, projectId: string): Promise<void> {
    try {
      await projectsApi.remove(organizationId, projectId)
      await fetch(organizationId)
      toast.success('Project deleted')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  return { projects, loading, reset, fetch, create, update, remove }
})
