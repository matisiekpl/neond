import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { toast } from 'vue-sonner'
import { authApi } from '@/api/auth'
import { getAppError } from '@/api/utils'
import { useOrganizationStore } from '@/stores/organization.store'
import type { User } from '@/types/models/user'

export const ACCESS_TOKEN = 'ACCESS_TOKEN'

export const useAuthStore = defineStore('auth', () => {
  const router = useRouter()
  const user = ref<User | undefined>(undefined)
  const initialized = ref(false)
  const loading = ref(false)
  const registrationOpen = ref(false)

  async function refreshUser(): Promise<void> {
    const token = localStorage.getItem(ACCESS_TOKEN)
    if (!token) {
      user.value = undefined
      return
    }
    try {
      user.value = await authApi.me()
    } catch {
      localStorage.removeItem(ACCESS_TOKEN)
      user.value = undefined
    }
  }

  async function bootstrap(): Promise<void> {
    const setup = await authApi.setup()
    registrationOpen.value = setup.registration_open
    await refreshUser()
    initialized.value = true
  }

  async function login(email: string, password: string): Promise<void> {
    try {
      loading.value = true
      const token = await authApi.login(email, password)
      localStorage.setItem(ACCESS_TOKEN, token)
      await refreshUser()
      const organizationStore = useOrganizationStore()
      await organizationStore.load()
      const organizationId = organizationStore.selectedOrganizationId
      if (organizationId) {
        await router.push({ name: 'projects.list', params: { organizationId } })
      } else {
        await router.push({ name: 'setup-organization' })
      }
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      loading.value = false
    }
  }

  async function register(name: string, email: string, password: string): Promise<void> {
    try {
      loading.value = true
      const token = await authApi.register(name, email, password)
      localStorage.setItem(ACCESS_TOKEN, token)
      await refreshUser()
      const organizationStore = useOrganizationStore()
      await organizationStore.load()
      const organizationId = organizationStore.selectedOrganizationId
      if (organizationId) {
        await router.push({ name: 'projects.list', params: { organizationId } })
      } else {
        await router.push({ name: 'setup-organization' })
      }
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      loading.value = false
    }
  }

  function logout(): void {
    localStorage.removeItem(ACCESS_TOKEN)
    user.value = undefined
    router.push('/login')
  }

  return { user, initialized, loading, registrationOpen, bootstrap, refreshUser, login, register, logout }
})