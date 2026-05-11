import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { logsApi, type DaemonLogComponent } from '@/api/logs'

export interface LogLine {
  timestamp: string
  stream: 'stdout' | 'stderr'
  message: string
}

const MAX_LINES = 5000

export const useLogsStore = defineStore('logs', () => {
  const lines = ref<LogLine[]>([])
  const connected = ref(false)
  let source: EventSource | null = null

  function handleMessage(event: MessageEvent) {
    const line = JSON.parse(event.data) as LogLine
    lines.value.push(line)
    if (lines.value.length > MAX_LINES) {
      lines.value = lines.value.slice(-MAX_LINES)
    }
  }

  function handleError() {
    connected.value = false
    toast.error('Log stream disconnected')
  }

  function open(eventSource: EventSource) {
    stop()
    source = eventSource
    source.onopen = () => {
      connected.value = true
    }
    source.onmessage = handleMessage
    source.onerror = handleError
  }

  function startDaemonLogs(component: DaemonLogComponent) {
    open(logsApi.streamDaemonLogs(component))
  }

  function startEndpointLogs(organizationId: string, projectId: string, branchId: string) {
    open(logsApi.streamEndpointLogs(organizationId, projectId, branchId))
  }

  function startPgbouncerLogs(organizationId: string, projectId: string, branchId: string) {
    open(logsApi.streamPgbouncerLogs(organizationId, projectId, branchId))
  }

  function stop() {
    source?.close()
    source = null
    lines.value = []
    connected.value = false
  }

  return { lines, connected, startDaemonLogs, startEndpointLogs, startPgbouncerLogs, stop }
})