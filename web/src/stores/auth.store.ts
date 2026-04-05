import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { toast } from 'vue-sonner'
import { authApi } from '@/api/auth'
import { getAppError } from '@/api/utils'
import type { User } from '@/types/models/user'

export const ACCESS_TOKEN = 'ACCESS_TOKEN'

export const useAuthStore = defineStore('auth', () => {
  const router = useRouter()
  const user = ref<User | undefined>(undefined)
  const initialized = ref(false)
  const loading = ref(false)

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
    await refreshUser()
    initialized.value = true
  }

  async function login(email: string, password: string): Promise<void> {
    try {
      loading.value = true
      const token = await authApi.login(email, password)
      localStorage.setItem(ACCESS_TOKEN, token)
      await refreshUser()
      await router.push('/dashboard')
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
      await router.push('/dashboard')
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

  return { user, initialized, loading, bootstrap, refreshUser, login, register, logout }
})
