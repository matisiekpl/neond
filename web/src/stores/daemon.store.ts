import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useIntervalFn } from '@vueuse/core'
import { daemonApi } from '@/api/daemon'
import type { DaemonState } from '@/types/models/daemon'
import { toast } from 'vue-sonner'
import { getAppError } from '@/api/utils'

export const useDaemonStore = defineStore('daemon', () => {
  const state = ref<DaemonState | null>(null)
  const loading = ref(false)
  const shuttingDownSubmitting = ref(false)
  const cancellingSubmitting = ref(false)

  const { pause, resume } = useIntervalFn(fetch, 1000, { immediate: false })

  async function fetch() {
    loading.value = true
    try {
      state.value = await daemonApi.get()
    } finally {
      loading.value = false
    }
  }

  async function shutdown(waitForCheckpoints: boolean) {
    try {
      shuttingDownSubmitting.value = true
      await daemonApi.shutdown(waitForCheckpoints)
      await fetch()
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      shuttingDownSubmitting.value = false
    }
  }

  async function cancelShutdown() {
    try {
      cancellingSubmitting.value = true
      await daemonApi.cancelShutdown()
      await fetch()
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      cancellingSubmitting.value = false
    }
  }

  function startPolling() {
    fetch()
    resume()
  }

  function stopPolling() {
    pause()
  }

  return { state, loading, shuttingDownSubmitting, cancellingSubmitting, startPolling, stopPolling, shutdown, cancelShutdown }
})