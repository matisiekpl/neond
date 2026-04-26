<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useTitle } from '@vueuse/core'
import { useRoute } from 'vue-router'
import { useOrganizationStore } from '@/stores/organization.store'
import { useLogsStore } from '@/stores/logs.store'
import LogsTextarea from '@/elements/LogsTextarea.vue'

useTitle('Compute endpoint logs — neond')

const route = useRoute()
const organizationStore = useOrganizationStore()
const logsStore = useLogsStore()

const organizationId = computed(() => organizationStore.selectedOrganizationId!)
const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)

onMounted(() => {
  logsStore.startEndpointLogs(organizationId.value, projectId.value, branchId.value)
})

onUnmounted(() => {
  logsStore.stop()
})
</script>

<template>
  <div class="flex h-full flex-col">
    <LogsTextarea />
  </div>
</template>