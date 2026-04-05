import { create } from "zustand"
import { toast } from "sonner"
import { branchesApi } from "~/api/branches"
import { getAppError } from "~/lib/errors"
import type { Branch } from "~/types/models/branch"

type BranchState = {
  branches: Branch[]
  loading: boolean
  reset: () => void
  fetchBranches: (organizationId: string, projectId: string, silent?: boolean) => Promise<void>
  createBranch: (organizationId: string, projectId: string, name: string, parentBranchId?: string) => Promise<Branch>
  renameBranch: (organizationId: string, projectId: string, branchId: string, name: string) => Promise<void>
  startEndpoint: (organizationId: string, projectId: string, branchId: string) => Promise<void>
  stopEndpoint: (organizationId: string, projectId: string, branchId: string) => Promise<void>
  deleteBranch: (organizationId: string, projectId: string, branchId: string) => Promise<void>
}

export const useBranchStore = create<BranchState>((set, get) => ({
  branches: [],
  loading: false,

  reset: () => set({ branches: [], loading: false }),

  fetchBranches: async (organizationId, projectId, silent = false) => {
    if (!silent) set({ loading: true })
    try {
      const list = await branchesApi.list(organizationId, projectId)
      set({ branches: list })
    } finally {
      if (!silent) set({ loading: false })
    }
  },

  createBranch: async (organizationId, projectId, name, parentBranchId?) => {
    try {
      const branch = await branchesApi.create(organizationId, projectId, name, parentBranchId)
      await get().fetchBranches(organizationId, projectId)
      toast.success("Branch created")
      return branch
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  renameBranch: async (organizationId, projectId, branchId, name) => {
    try {
      await branchesApi.rename(organizationId, projectId, branchId, name)
      await get().fetchBranches(organizationId, projectId)
      toast.success("Branch renamed")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  startEndpoint: async (organizationId, projectId, branchId) => {
    try {
      await branchesApi.launch(organizationId, projectId, branchId)
      await get().fetchBranches(organizationId, projectId)
      toast.success("Endpoint started")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  stopEndpoint: async (organizationId, projectId, branchId) => {
    try {
      await branchesApi.shutdown(organizationId, projectId, branchId)
      await get().fetchBranches(organizationId, projectId)
      toast.success("Endpoint stopped")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  deleteBranch: async (organizationId, projectId, branchId) => {
    try {
      await branchesApi.remove(organizationId, projectId, branchId)
      await get().fetchBranches(organizationId, projectId)
      toast.success("Branch deleted")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },
}))
