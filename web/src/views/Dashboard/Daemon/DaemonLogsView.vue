<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useTitle } from '@vueuse/core'
import { useRoute } from 'vue-router'
import { useLogsStore } from '@/stores/logs.store'
import { type DaemonLogComponent } from '@/api/logs'
import LogsTextarea from '@/elements/LogsTextarea.vue'

const route = useRoute()

useTitle(computed(() => `Daemon — Logs — ${route.params.component} — neond`))
const logsStore = useLogsStore()

function start() {
  logsStore.startDaemonLogs(route.params.component as DaemonLogComponent)
}

onMounted(start)

watch(() => route.params.component, start)

onUnmounted(() => {
  logsStore.stop()
})
</script>

<template>
  <div class="flex flex-col w-[calc(100vw-2rem)] md:w-[calc(100vw-19rem)]">
    <LogsTextarea />
  </div>
</template>