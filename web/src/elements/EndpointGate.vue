<script setup lang="ts">
import { computed } from 'vue'
import { Database, Play, Loader2 } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { useBranchStore } from '@/stores/branch.store'

const props = defineProps<{
  organizationId: string
  projectId: string
  branchId: string
}>()

const branchStore = useBranchStore()

const branch = computed(() => branchStore.branches.find((b) => b.id === props.branchId))
const endpointStatus = computed(() => branch.value?.endpoint_status)
const isRunning = computed(() => endpointStatus.value === 'running')
const isTransient = computed(
  () => endpointStatus.value === 'starting' || endpointStatus.value === 'stopping',
)

async function startEndpoint() {
  await branchStore.launchEndpoint(props.organizationId, props.projectId, props.branchId)
}
</script>

<template>
  <slot v-if="isRunning" />

  <div v-else class="h-full flex items-center justify-center">
    <div class="max-w-md flex flex-col items-center text-center gap-3 p-6">
      <Database class="size-10 text-muted-foreground" />
      <div class="text-base font-medium">
        <template v-if="endpointStatus === 'starting'">Endpoint is starting…</template>
        <template v-else-if="endpointStatus === 'stopping'">Endpoint is stopping…</template>
        <template v-else-if="endpointStatus === 'failed'">Endpoint failed to start</template>
        <template v-else>Compute endpoint is stopped</template>
      </div>
      <p class="text-sm text-muted-foreground">
        To browse data on this branch you need to start the compute endpoint.
      </p>
      <Button
        v-if="endpointStatus === 'stopped' || endpointStatus === 'failed'"
        class="cursor-pointer"
        @click="startEndpoint"
      >
        <Play class="size-4" />
        Start endpoint
      </Button>
      <Loader2 v-else-if="isTransient" class="size-5 animate-spin text-muted-foreground" />
    </div>
  </div>
</template>