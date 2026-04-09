import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { organizationsApi } from '@/api/organizations'
import { membersApi } from '@/api/members'
import { getAppError } from '@/api/utils'
import type { Organization, OrganizationMemberUser } from '@/types/models/organization'

export const CURRENT_ORG_STORAGE_KEY = 'neond_current_org_id'

export const useOrganizationStore = defineStore('organization', () => {
  const organizations = ref<Organization[]>([])
  const selectedOrganizationId = ref<string | null>(null)
  const loaded = ref(false)
  const loading = ref(false)
  const members = ref<OrganizationMemberUser[]>([])
  const membersLoading = ref(false)

  function reset(): void {
    organizations.value = []
    selectedOrganizationId.value = null
    loaded.value = false
    loading.value = false
    members.value = []
    membersLoading.value = false
  }

  function saveSelected(id: string | null): void {
    if (id) {
      localStorage.setItem(CURRENT_ORG_STORAGE_KEY, id)
    } else {
      localStorage.removeItem(CURRENT_ORG_STORAGE_KEY)
    }
    selectedOrganizationId.value = id
  }

  function initSelection(orgs: Organization[]): void {
    const stored = localStorage.getItem(CURRENT_ORG_STORAGE_KEY)
    const ids = new Set(orgs.map((o) => o.id))
    if (stored && ids.has(stored)) {
      selectedOrganizationId.value = stored
      return
    }
    const first = orgs[0]
    if (first) {
      saveSelected(first.id)
    } else {
      saveSelected(null)
    }
  }

  async function fetch(): Promise<Organization[]> {
    loading.value = true
    try {
      const list = await organizationsApi.list()
      organizations.value = list
      return list
    } finally {
      loading.value = false
    }
  }

  async function load(): Promise<void> {
    const orgs = await fetch()
    initSelection(orgs)
    loaded.value = true
  }

  async function create(name: string): Promise<Organization> {
    const org = await organizationsApi.create(name)
    await fetch()
    saveSelected(org.id)
    toast.success('Organization created')
    return org
  }

  async function update(organizationId: string, name: string): Promise<Organization> {
    const org = await organizationsApi.update(organizationId, name)
    await fetch()
    toast.success('Organization updated')
    return org
  }

  async function remove(organizationId: string): Promise<void> {
    await organizationsApi.remove(organizationId)
    const orgs = await fetch()
    initSelection(orgs)
    toast.success('Organization deleted')
  }

  async function fetchMembers(organizationId: string): Promise<void> {
    membersLoading.value = true
    try {
      members.value = await membersApi.list(organizationId)
    } finally {
      membersLoading.value = false
    }
  }

  async function assignMemberByEmail(organizationId: string, email: string): Promise<void> {
    try {
      await membersApi.assignByEmail(organizationId, email)
      await fetchMembers(organizationId)
      toast.success('Member added')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function revokeMember(organizationId: string, userId: string): Promise<void> {
    try {
      await membersApi.remove(organizationId, userId)
      await fetchMembers(organizationId)
      toast.success('Member removed')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  return {
    organizations,
    selectedOrganizationId,
    loaded,
    loading,
    members,
    membersLoading,
    reset,
    saveSelected,
    initSelection,
    fetch,
    load,
    create,
    update,
    remove,
    fetchMembers,
    assignMemberByEmail,
    revokeMember,
  }
})
