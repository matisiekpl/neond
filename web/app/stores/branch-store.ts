import { create } from "zustand"
import { toast } from "sonner"
import { branchesApi } from "~/api/branches"
import { getAppError } from "~/lib/errors"
import type { Branch } from "~/types/models/branch"

type BranchState = {
  branches: Branch[]
  loading: boolean
  reset: () => void
  fetchBranches: (orgId: string, projectId: string, silent?: boolean) => Promise<void>
  createBranch: (orgId: string, projectId: string, name: string, parentBranchId?: string) => Promise<Branch>
  renameBranch: (orgId: string, projectId: string, branchId: string, name: string) => Promise<void>
  startEndpoint: (orgId: string, projectId: string, branchId: string) => Promise<void>
  stopEndpoint: (orgId: string, projectId: string, branchId: string) => Promise<void>
  deleteBranch: (orgId: string, projectId: string, branchId: string) => Promise<void>
}

export const useBranchStore = create<BranchState>((set, get) => ({
  branches: [],
  loading: false,

  reset: () => set({ branches: [], loading: false }),

  fetchBranches: async (orgId, projectId, silent = false) => {
    if (!silent) set({ loading: true })
    try {
      const list = await branchesApi.list(orgId, projectId)
      set({ branches: list })
    } finally {
      if (!silent) set({ loading: false })
    }
  },

  createBranch: async (orgId, projectId, name, parentBranchId?) => {
    try {
      const branch = await branchesApi.create(orgId, projectId, name, parentBranchId)
      await get().fetchBranches(orgId, projectId)
      toast.success("Branch created")
      return branch
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  renameBranch: async (orgId, projectId, branchId, name) => {
    try {
      await branchesApi.rename(orgId, projectId, branchId, name)
      await get().fetchBranches(orgId, projectId)
      toast.success("Branch renamed")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  startEndpoint: async (orgId, projectId, branchId) => {
    try {
      await branchesApi.launch(orgId, projectId, branchId)
      await get().fetchBranches(orgId, projectId)
      toast.success("Endpoint started")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  stopEndpoint: async (orgId, projectId, branchId) => {
    try {
      await branchesApi.shutdown(orgId, projectId, branchId)
      await get().fetchBranches(orgId, projectId)
      toast.success("Endpoint stopped")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  deleteBranch: async (orgId, projectId, branchId) => {
    try {
      await branchesApi.remove(orgId, projectId, branchId)
      await get().fetchBranches(orgId, projectId)
      toast.success("Branch deleted")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },
}))
