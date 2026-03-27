import {defineStore} from 'pinia'
import {computed, ref} from 'vue'
import {membersApi} from '@/api/members.ts'
import {organizationsApi} from '@/api/organizations.ts'
import {getAppError} from '@/api/utils.ts'
import {toast} from 'vue-sonner'
import type {Organization} from '@/types/models/organization.ts'
import type {OrganizationMemberUser} from '@/types/models/organizationMemberUser.ts'

export const CURRENT_ORG_STORAGE_KEY = 'neond_current_org_id'

export const useOrganizationStore = defineStore('organization', () => {
  const organizations = ref<Organization[]>([])
  const selectedOrganizationId = ref<string | null>(null)
  const loading = ref(false)
  const members = ref<OrganizationMemberUser[]>([])
  const membersLoading = ref(false)

  const currentOrganization = computed(() => {
    if (!selectedOrganizationId.value) return undefined
    return organizations.value.find((o) => o.id === selectedOrganizationId.value)
  })

  function saveSelectedOrganization(id: string | null) {
    if (id) {
      localStorage.setItem(CURRENT_ORG_STORAGE_KEY, id)
    } else {
      localStorage.removeItem(CURRENT_ORG_STORAGE_KEY)
    }
    selectedOrganizationId.value = id
  }

  function init() {
    const stored = localStorage.getItem(CURRENT_ORG_STORAGE_KEY)
    const ids = new Set(organizations.value.map((o) => o.id))
    if (stored && ids.has(stored)) {
      selectedOrganizationId.value = stored
      return
    }
    const first = organizations.value[0]
    if (first) {
      saveSelectedOrganization(first.id)
    } else {
      saveSelectedOrganization(null)
    }
  }

  async function fetchOrganizations() {
    loading.value = true
    try {
      organizations.value = await organizationsApi.list()
    } finally {
      loading.value = false
    }
  }

  async function ensureOrganizations(userName: string) {
    await fetchOrganizations()
    if (organizations.value.length === 0) {
      const personalName = `${userName}'s organization`
      await organizationsApi.create(personalName)
      await fetchOrganizations()
    }
    init()
  }

  async function createOrganization(name: string) {
    const org = await organizationsApi.create(name)
    await fetchOrganizations()
    saveSelectedOrganization(org.id)
    toast.success('Organization created')
    return org
  }

  async function updateOrganization(orgId: string, name: string) {
    const org = await organizationsApi.update(orgId, name)
    await fetchOrganizations()
    toast.success('Organization updated')
    return org
  }

  async function deleteOrganization(orgId: string) {
    await organizationsApi.remove(orgId)
    await fetchOrganizations()
    init()
    toast.success('Organization deleted')
  }

  async function fetchMembers(orgId: string) {
    membersLoading.value = true
    try {
      members.value = await membersApi.list(orgId)
    } finally {
      membersLoading.value = false
    }
  }

  async function addMemberByEmail(orgId: string, email: string) {
    try {
      await membersApi.addByEmail(orgId, email)
      await fetchMembers(orgId)
      toast.success('Member added')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  async function removeMember(orgId: string, userId: string) {
    try {
      await membersApi.remove(orgId, userId)
      await fetchMembers(orgId)
      toast.success('Member removed')
    } catch (e) {
      toast.error(getAppError(e))
      throw e
    }
  }

  return {
    organizations,
    selectedOrganizationId,
    currentOrganization,
    loading,
    members,
    membersLoading,
    fetchOrganizations,
    ensureOrganizations,
    saveSelectedOrganization,
    createOrganization,
    updateOrganization,
    deleteOrganization,
    fetchMembers,
    addMemberByEmail,
    removeMember,
  }
})
