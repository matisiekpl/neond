import { create } from "zustand"
import type { NavigateFunction } from "react-router"
import { toast } from "sonner"
import { authApi } from "~/api/auth"
import { ACCESS_TOKEN } from "~/lib/constants"
import { getAppError } from "~/lib/errors"
import type { User } from "~/types/models"

type AuthState = {
  navigate: NavigateFunction | null
  user: User | undefined
  initialized: boolean
  loading: boolean
  setNavigate: (nav: NavigateFunction | null) => void
  bootstrap: () => Promise<void>
  refreshUser: () => Promise<void>
  login: (email: string, password: string) => Promise<void>
  register: (name: string, email: string, password: string) => Promise<void>
  logout: () => void
}

export const useAuthStore = create<AuthState>((set, get) => ({
  navigate: null,
  user: undefined,
  initialized: false,
  loading: false,

  setNavigate: (nav) => set({ navigate: nav }),

  refreshUser: async () => {
    const token = localStorage.getItem(ACCESS_TOKEN)
    if (!token) {
      set({ user: undefined })
      return
    }
    try {
      const u = await authApi.me()
      set({ user: u })
    } catch {
      localStorage.removeItem(ACCESS_TOKEN)
      set({ user: undefined })
    }
  },

  bootstrap: async () => {
    await get().refreshUser()
    set({ initialized: true })
  },

  login: async (email, password) => {
    const { navigate, refreshUser } = get()
    try {
      set({ loading: true })
      const token = await authApi.login(email, password)
      localStorage.setItem(ACCESS_TOKEN, token)
      await refreshUser()
      navigate?.("/dashboard")
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      set({ loading: false })
    }
  },

  register: async (name, email, password) => {
    const { navigate, refreshUser } = get()
    try {
      set({ loading: true })
      const token = await authApi.register(name, email, password)
      localStorage.setItem(ACCESS_TOKEN, token)
      await refreshUser()
      navigate?.("/dashboard")
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      set({ loading: false })
    }
  },

  logout: () => {
    localStorage.removeItem(ACCESS_TOKEN)
    set({ user: undefined })
    get().navigate?.("/login")
  },
}))
