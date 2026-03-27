import {defineStore} from 'pinia'
import {ref} from 'vue'
import {authApi} from '@/api/auth.ts'
import {getAppError} from '@/api/utils.ts'
import {toast} from 'vue-sonner'
import {useRouter, useRoute} from 'vue-router'
import type {User} from '@/types/models/user.ts'

export const ACCESS_TOKEN = 'ACCESS_TOKEN'
export const RETURN_URL = 'RETURN_URL'

export const useAuthStore = defineStore('auth', () => {
  const name = ref<string>('')
  const email = ref<string>('')
  const password = ref<string>('')
  const router = useRouter()
  const route = useRoute()
  const loading = ref(false)
  const user = ref<User | undefined>(undefined)
  const initialized = ref(false)

  async function check() {
    if (localStorage.getItem(ACCESS_TOKEN)) {
      await router.push('/')
    }
    if (route.query.email) {
      email.value = route.query.email as string
    }
  }

  async function login() {
    try {
      loading.value = true
      const token = await authApi.login(email.value, password.value)
      localStorage.setItem(ACCESS_TOKEN, token)
      await init()
      const returnUrl = localStorage.getItem(RETURN_URL)
        || new URLSearchParams(window.location.search).get('return')
        || '/'
      localStorage.removeItem(RETURN_URL)
      await router.push(returnUrl)
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      loading.value = false
    }
  }

  async function register() {
    try {
      loading.value = true
      const token = await authApi.register(name.value, email.value, password.value)
      localStorage.setItem(ACCESS_TOKEN, token)
      await init()
      const returnUrl = localStorage.getItem(RETURN_URL)
        || new URLSearchParams(window.location.search).get('return')
        || '/'
      localStorage.removeItem(RETURN_URL)
      await router.push(returnUrl)
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      loading.value = false
    }
  }

  async function logout() {
    localStorage.removeItem(ACCESS_TOKEN)
    await router.push('/login')
  }

  async function init() {
    user.value = await authApi.me()
    initialized.value = true
  }

  return {check, login, register, logout, init, name, email, password, loading, user, initialized}
})
