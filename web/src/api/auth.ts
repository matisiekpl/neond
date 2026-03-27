import api from '@/api/base.ts'
import type {AuthResponse} from '@/types/dto/authResponse.ts'
import type {User} from '@/types/models/user.ts'

async function login(email: string, password: string): Promise<string> {
  const response = await api.post('auth/login', {email, password})
  return (response.data as AuthResponse).token
}

async function register(name: string, email: string, password: string): Promise<string> {
  const response = await api.post('auth/register', {name, email, password})
  return (response.data as AuthResponse).token
}

async function me(): Promise<User> {
  const response = await api.get('auth/me')
  return response.data as User
}

export const authApi = {
  login,
  register,
  me,
}
