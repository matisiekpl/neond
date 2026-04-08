import api from '@/api/base'
import type { DaemonState } from '@/types/models/daemon'

export const daemonApi = {
  async get(): Promise<DaemonState> {
    const response = await api.get<DaemonState>('daemon')
    return response.data
  },

  async shutdown(waitForCheckpoints: boolean): Promise<void> {
    await api.post('daemon/shutdown', { wait_for_checkpoints: waitForCheckpoints })
  },

  async cancelShutdown(): Promise<void> {
    await api.delete('daemon/shutdown')
  },
}