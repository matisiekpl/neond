import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { usersApi } from '@/api/users'
import { getAppError } from '@/api/utils'
import type { User } from '@/types/models/user'

export const useUsersStore = defineStore('users', () => {
  const users = ref<User[]>([])
  const loading = ref(false)

  async function fetch(): Promise<void> {
    loading.value = true
    try {
      users.value = await usersApi.list()
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      loading.value = false
    }
  }

  async function create(name: string, email: string, password: string): Promise<void> {
    try {
      await usersApi.create(name, email, password)
      await fetch()
    } catch (err) {
      toast.error(getAppError(err))
    }
  }

  async function update(userId: string, data: { name: string; email: string; is_admin: boolean }): Promise<void> {
    try {
      await usersApi.update(userId, data)
      await fetch()
    } catch (err) {
      toast.error(getAppError(err))
    }
  }

  async function resetPassword(userId: string, password: string): Promise<void> {
    try {
      await usersApi.resetPassword(userId, password)
      toast.success('Password reset successfully')
    } catch (err) {
      toast.error(getAppError(err))
    }
  }

  async function remove(userId: string): Promise<void> {
    try {
      await usersApi.remove(userId)
      await fetch()
    } catch (err) {
      toast.error(getAppError(err))
    }
  }

  return { users, loading, fetch, create, update, resetPassword, remove }
})
