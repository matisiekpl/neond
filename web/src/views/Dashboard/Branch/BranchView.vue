<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTitle } from '@vueuse/core'
import { useProjectStore } from '@/stores/project.store'
import { useBranchStore } from '@/stores/branch.store'
import { useOrganizationStore } from '@/stores/organization.store'
import EndpointStatusBadge from '@/elements/EndpointStatusBadge.vue'

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const branchStore = useBranchStore()
const organizationStore = useOrganizationStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)

const project = computed(() => projectStore.projects.find((p) => p.id === projectId.value))
const branch = computed(() => branchStore.branches.find((b) => b.id === branchId.value))

useTitle(
  computed(() =>
    project.value && branch.value
      ? `${project.value.name} / ${branch.value.name} — neond`
      : 'neond',
  ),
)
</script>

<template>
  <div
    v-if="!branch"
    class="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center"
  >
    <p class="text-sm font-medium">Branch not found</p>
    <p class="mt-1 text-sm text-muted-foreground">
      This branch may have been deleted or you don't have access to it.
    </p>
    <button
      type="button"
      class="mt-4 text-sm underline underline-offset-4"
      @click="router.push({ name: 'projects.show', params: { organizationId: organizationStore.selectedOrganizationId, projectId } })"
    >
      Back to project
    </button>
  </div>

  <div v-else class="space-y-6">
    <div>
      <div class="flex items-center gap-2">
        <h1 class="text-lg font-semibold">{{ branch.name }}</h1>
        <EndpointStatusBadge :status="branch.endpoint_status" />
      </div>
      <div class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted-foreground">
        <span class="font-mono text-xs">{{ branch.id }}</span>
      </div>
    </div>

    <div class="border-b">
      <nav class="flex gap-0">
        <RouterLink
          :to="{ name: 'projects.branches.data', params: { organizationId: organizationStore.selectedOrganizationId, projectId, branchId } }"
          class="px-4 py-2 text-sm font-medium"
          :class="route.name === 'projects.branches.data' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground hover:text-foreground'"
        >
          Data
        </RouterLink>
      </nav>
    </div>

    <RouterView />
  </div>
</template>