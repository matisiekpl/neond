import api from '@/api/base'
import type { AuthResponse } from '@/types/dto/authResponse'
import type { SetupResponse } from '@/types/dto/setupResponse'
import type { User } from '@/types/models/user'

export const authApi = {
  async setup(): Promise<SetupResponse> {
    const response = await api.get<SetupResponse>('auth/setup')
    return response.data
  },

  async login(email: string, password: string): Promise<string> {
    const response = await api.post<AuthResponse>('auth/login', { email, password })
    return response.data.token
  },

  async register(name: string, email: string, password: string): Promise<string> {
    const response = await api.post<AuthResponse>('auth/register', { name, email, password })
    return response.data.token
  },

  async me(): Promise<User> {
    const response = await api.get<User>('auth/me')
    return response.data
  },
}
