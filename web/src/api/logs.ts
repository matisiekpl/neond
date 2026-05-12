import { Config } from '@/config'
import { ACCESS_TOKEN } from '@/stores/auth.store'

export type DaemonLogComponent =
  | 'storage_broker'
  | 'storage_controller'
  | 'pageserver'
  | 'safekeeper'
  | 'storage_controller_db'
  | 'management_db'

export const logsApi = {
  streamDaemonLogs(component: DaemonLogComponent): EventSource {
    const token = localStorage.getItem(ACCESS_TOKEN) ?? ''
    return new EventSource(
      `${Config.serverUrl}/api/daemon/logs/${component}?token=${encodeURIComponent(token)}`,
    )
  },

  streamEndpointLogs(organizationId: string, projectId: string, branchId: string): EventSource {
    const token = localStorage.getItem(ACCESS_TOKEN) ?? ''
    return new EventSource(
      `${Config.serverUrl}/api/organizations/${organizationId}/projects/${projectId}/branches/${branchId}/endpoint/logs?token=${encodeURIComponent(token)}`,
    )
  },

  streamPgbouncerLogs(organizationId: string, projectId: string, branchId: string): EventSource {
    const token = localStorage.getItem(ACCESS_TOKEN) ?? ''
    return new EventSource(
      `${Config.serverUrl}/api/organizations/${organizationId}/projects/${projectId}/branches/${branchId}/endpoint/logs/pgbouncer?token=${encodeURIComponent(token)}`,
    )
  },
}