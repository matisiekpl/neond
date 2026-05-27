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
  const loading = ref(false)
  let source: EventSource | null = null

  function trimLines() {
    if (lines.value.length > MAX_LINES) {
      lines.value = lines.value.slice(-MAX_LINES)
    }
  }

  function handleMessage(event: MessageEvent) {
    loading.value = false
    const line = JSON.parse(event.data) as LogLine
    lines.value.push(line)
    trimLines()
  }

  function handleSnapshot(event: MessageEvent) {
    loading.value = false
    const snapshot = JSON.parse(event.data) as LogLine[]
    lines.value.push(...snapshot)
    trimLines()
  }

  function handleError() {
    loading.value = false
    connected.value = false
    toast.error('Log stream disconnected')
  }

  function open(eventSource: EventSource) {
    stop()
    source = eventSource
    loading.value = true
    source.onopen = () => {
      connected.value = true
    }
    source.onmessage = handleMessage
    source.addEventListener('snapshot', handleSnapshot as EventListener)
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

  function startImportLogs(organizationId: string, projectId: string, branchId: string) {
    open(logsApi.streamImportLogs(organizationId, projectId, branchId))
  }

  function stop() {
    source?.close()
    source = null
    lines.value = []
    connected.value = false
    loading.value = false
  }

  return { lines, connected, loading, startDaemonLogs, startEndpointLogs, startPgbouncerLogs, startImportLogs, stop }
})