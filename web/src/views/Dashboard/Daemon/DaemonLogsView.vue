<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import { useTitle } from '@vueuse/core'
import { useRoute } from 'vue-router'
import { useLogsStore } from '@/stores/logs.store'
import { type DaemonLogComponent } from '@/api/logs'
import LogsTextarea from '@/elements/LogsTextarea.vue'

useTitle('Daemon logs — neond')

const route = useRoute()
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
  <div class="flex h-full flex-col">
    <LogsTextarea />
  </div>
</template>