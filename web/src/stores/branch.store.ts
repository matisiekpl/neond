import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { branchesApi } from '@/api/branches'
import { getAppError } from '@/api/utils'
import type { Branch } from '@/types/models/branch'

export const useBranchStore = defineStore('branch', () => {
  const branches = ref<Branch[]>([])
  const loading = ref(false)

  function reset(): void {
    branches.value = []
    loading.value = false
  }

  async function fetch(organizationId: string, projectId: string, silent = false): Promise<void> {
    if (!silent) loading.value = true
    try {
      branches.value = await branchesApi.list(organizationId, projectId)
    } finally {
      if (!silent) loading.value = false
    }
  }

  async function create(organizationId: string, projectId: string, name: string, parentBranchId?: string): Promise<Branch> {
    try {
      const branch = await branchesApi.create(organizationId, projectId, name, parentBranchId)
      await fetch(organizationId, projectId)
      toast.success('Branch created')
      return branch
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function update(organizationId: string, projectId: string, branchId: string, name: string): Promise<void> {
    try {
      await branchesApi.update(organizationId, projectId, branchId, name)
      await fetch(organizationId, projectId)
      toast.success('Branch renamed')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function launchEndpoint(organizationId: string, projectId: string, branchId: string): Promise<void> {
    try {
      await branchesApi.launch(organizationId, projectId, branchId)
      await fetch(organizationId, projectId)
      toast.success('Endpoint started')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function shutdownEndpoint(organizationId: string, projectId: string, branchId: string): Promise<void> {
    try {
      await branchesApi.shutdown(organizationId, projectId, branchId)
      await fetch(organizationId, projectId)
      toast.success('Endpoint stopped')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function remove(organizationId: string, projectId: string, branchId: string): Promise<void> {
    try {
      await branchesApi.remove(organizationId, projectId, branchId)
      await fetch(organizationId, projectId)
      toast.success('Branch deleted')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  return { branches, loading, reset, fetch, create, update, launchEndpoint, shutdownEndpoint, remove }
})
