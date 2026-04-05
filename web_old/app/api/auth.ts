import type { User } from "~/types/models/user"
import api from "./client"

interface AuthResponse {
  token: string
}

export const authApi = {
  async login(email: string, password: string): Promise<string> {
    const response = await api.post<AuthResponse>("auth/login", { email, password })
    return response.data.token
  },

  async register(
    name: string,
    email: string,
    password: string,
  ): Promise<string> {
    const response = await api.post<AuthResponse>("auth/register", {
      name,
      email,
      password,
    })
    return response.data.token
  },

  async me(): Promise<User> {
    const response = await api.get<User>("auth/me")
    return response.data
  },
}
