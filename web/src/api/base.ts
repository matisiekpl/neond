import axios, {type InternalAxiosRequestConfig} from 'axios'
import {Config} from '@/config.ts'
import {jwtDecode} from 'jwt-decode'
import {ACCESS_TOKEN} from '@/stores/auth.store.ts'

const api = axios.create({
  baseURL: Config.serverUrl + '/api/',
})

api.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    const token = localStorage.getItem(ACCESS_TOKEN)
    if (token) {
      const decoded = jwtDecode(token) as {exp: number}
      const currentTime = Date.now() / 1000
      if (decoded && decoded.exp < currentTime) {
        localStorage.removeItem(ACCESS_TOKEN)
        window.location.href = '/login'
      }
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
)

export default api
