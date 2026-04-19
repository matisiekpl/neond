import api from '@/api/base'
import type { User } from '@/types/models/user'

async function list(): Promise<User[]> {
  const response = await api.get<User[]>('auth/users')
  return response.data
}

async function create(name: string, email: string, password: string): Promise<User> {
  const response = await api.post<User>('auth/users', { name, email, password })
  return response.data
}

async function update(userId: string, data: { name: string; email: string; is_admin: boolean }): Promise<User> {
  const response = await api.put<User>(`auth/users/${userId}`, data)
  return response.data
}

async function resetPassword(userId: string, password: string): Promise<void> {
  await api.put(`auth/users/${userId}/password`, { password })
}

async function remove(userId: string): Promise<void> {
  await api.delete(`auth/users/${userId}`)
}

export const usersApi = {
  list,
  create,
  update,
  resetPassword,
  remove,
}
