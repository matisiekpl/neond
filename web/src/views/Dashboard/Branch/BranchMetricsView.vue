<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import MetricsDashboard from '@/elements/MetricsDashboard.vue'
import { useOrganizationStore } from '@/stores/organization.store'
import { useMetricStore } from '@/stores/metric.store'
import { metricCharts } from '@/lib/metricPresets'

const route = useRoute()
const organizationStore = useOrganizationStore()
const metricStore = useMetricStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)
const organizationId = computed(() => organizationStore.selectedOrganizationId)

onMounted(() => {
  if (organizationId.value) {
    metricStore.startBranchPolling(organizationId.value, projectId.value, branchId.value)
  }
})

onUnmounted(() => {
  metricStore.reset()
})

watch(
  () => [organizationId.value, projectId.value, branchId.value] as const,
  ([nextOrganizationId, nextProjectId, nextBranchId]) => {
    if (nextOrganizationId) {
      metricStore.startBranchPolling(nextOrganizationId, nextProjectId, nextBranchId)
    }
  },
)
</script>

<template>
  <MetricsDashboard v-if="organizationId" :charts="metricCharts" />
</template>