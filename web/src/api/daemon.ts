import api from '@/api/base'
import type { DaemonState } from '@/types/models/daemon'

export const daemonApi = {
  async get(): Promise<DaemonState> {
    const response = await api.get<DaemonState>('daemon')
    return response.data
  },
}