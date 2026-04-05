import { defineStore } from 'pinia'
import { ref } from 'vue'
import { daemonApi } from '@/api/daemon'
import type { DaemonState } from '@/types/models/daemon'

export const useDaemonStore = defineStore('daemon', () => {
  const state = ref<DaemonState | null>(null)
  const loading = ref(false)
  let intervalId: ReturnType<typeof setInterval> | null = null

  async function fetch() {
    loading.value = true
    try {
      state.value = await daemonApi.get()
    } finally {
      loading.value = false
    }
  }

  function startPolling() {
    fetch()
    intervalId = setInterval(fetch, 1000)
  }

  function stopPolling() {
    if (intervalId !== null) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  return { state, loading, startPolling, stopPolling }
})