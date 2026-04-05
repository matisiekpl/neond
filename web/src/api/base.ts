import axios, { type InternalAxiosRequestConfig } from 'axios'
import { jwtDecode } from 'jwt-decode'
import { Config } from '@/config'
import { ACCESS_TOKEN } from '@/stores/auth.store'

const api = axios.create({
  baseURL: `${Config.serverUrl}/api/`,
})

api.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  const token = localStorage.getItem(ACCESS_TOKEN)
  if (token) {
    try {
      const decoded = jwtDecode<{ exp: number }>(token)
      const currentTime = Date.now() / 1000
      if (decoded.exp < currentTime) {
        localStorage.removeItem(ACCESS_TOKEN)
        window.location.href = '/login'
      } else {
        config.headers.Authorization = `Bearer ${token}`
      }
    } catch {
      localStorage.removeItem(ACCESS_TOKEN)
    }
  }
  return config
})

export default api
