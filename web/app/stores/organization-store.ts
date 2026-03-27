import { create } from "zustand"
import { toast } from "sonner"
import { membersApi } from "~/api/members"
import { organizationsApi } from "~/api/organizations"
import { CURRENT_ORG_STORAGE_KEY } from "~/lib/constants"
import { getAppError } from "~/lib/errors"
import type { Organization, OrganizationMemberUser } from "~/types/models/organization"

type OrganizationState = {
  organizations: Organization[]
  selectedOrganizationId: string | null
  loaded: boolean
  loading: boolean
  members: OrganizationMemberUser[]
  membersLoading: boolean
  reset: () => void
  saveSelectedOrganization: (id: string | null) => void
  initSelection: (orgs: Organization[]) => void
  fetchOrganizations: () => Promise<Organization[]>
  loadOrganizations: () => Promise<void>
  createOrganization: (name: string) => Promise<Organization>
  updateOrganization: (orgId: string, name: string) => Promise<Organization>
  deleteOrganization: (orgId: string) => Promise<void>
  fetchMembers: (orgId: string) => Promise<void>
  addMemberByEmail: (orgId: string, email: string) => Promise<void>
  removeMember: (orgId: string, userId: string) => Promise<void>
}

export const useOrganizationStore = create<OrganizationState>((set, get) => ({
  organizations: [],
  selectedOrganizationId: null,
  loaded: false,
  loading: false,
  members: [],
  membersLoading: false,

  reset: () =>
    set({
      organizations: [],
      selectedOrganizationId: null,
      loaded: false,
      members: [],
      loading: false,
      membersLoading: false,
    }),

  saveSelectedOrganization: (id) => {
    if (id) {
      localStorage.setItem(CURRENT_ORG_STORAGE_KEY, id)
    } else {
      localStorage.removeItem(CURRENT_ORG_STORAGE_KEY)
    }
    set({ selectedOrganizationId: id })
  },

  initSelection: (orgs) => {
    const stored = localStorage.getItem(CURRENT_ORG_STORAGE_KEY)
    const ids = new Set(orgs.map((o) => o.id))
    if (stored && ids.has(stored)) {
      set({ selectedOrganizationId: stored })
      return
    }
    const first = orgs[0]
    if (first) {
      get().saveSelectedOrganization(first.id)
    } else {
      get().saveSelectedOrganization(null)
    }
  },

  fetchOrganizations: async () => {
    set({ loading: true })
    try {
      const list = await organizationsApi.list()
      set({ organizations: list })
      return list
    } finally {
      set({ loading: false })
    }
  },

  loadOrganizations: async () => {
    const orgs = await get().fetchOrganizations()
    get().initSelection(orgs)
  },

  createOrganization: async (name) => {
    const org = await organizationsApi.create(name)
    await get().fetchOrganizations()
    get().saveSelectedOrganization(org.id)
    toast.success("Organization created")
    return org
  },

  updateOrganization: async (orgId, name) => {
    const org = await organizationsApi.update(orgId, name)
    await get().fetchOrganizations()
    toast.success("Organization updated")
    return org
  },

  deleteOrganization: async (orgId) => {
    await organizationsApi.remove(orgId)
    const orgs = await get().fetchOrganizations()
    get().initSelection(orgs)
    toast.success("Organization deleted")
  },

  fetchMembers: async (orgId) => {
    set({ membersLoading: true })
    try {
      const list = await membersApi.list(orgId)
      set({ members: list })
    } finally {
      set({ membersLoading: false })
    }
  },

  addMemberByEmail: async (orgId, email) => {
    try {
      await membersApi.addByEmail(orgId, email)
      await get().fetchMembers(orgId)
      toast.success("Member added")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },

  removeMember: async (orgId, userId) => {
    try {
      await membersApi.remove(orgId, userId)
      await get().fetchMembers(orgId)
      toast.success("Member removed")
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  },
}))
