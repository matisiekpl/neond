<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useTitle } from '@vueuse/core'
import { useRoute } from 'vue-router'
import { useOrganizationStore } from '@/stores/organization.store'
import { useLogsStore } from '@/stores/logs.store'
import LogsTextarea from '@/elements/LogsTextarea.vue'

const route = useRoute()
const organizationStore = useOrganizationStore()
const logsStore = useLogsStore()

const organizationId = computed(() => organizationStore.selectedOrganizationId!)
const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)
const component = computed(() => route.params.component as string)

useTitle(computed(() => `Logs — ${component.value} — neond`))

function start() {
  if (component.value === 'pgbouncer') {
    logsStore.startPgbouncerLogs(organizationId.value, projectId.value, branchId.value)
  } else {
    logsStore.startEndpointLogs(organizationId.value, projectId.value, branchId.value)
  }
}

onMounted(start)
watch(() => route.params.component, start)

onUnmounted(() => {
  logsStore.stop()
})
</script>

<template>
  <div class="flex flex-col w-[calc(100vw-2rem)] md:w-[calc(100vw-19rem)]">
    <LogsTextarea :compact="true" />
  </div>
</template>