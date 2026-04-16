<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useMediaQuery } from '@vueuse/core'
import { Database, Play, Loader2, ChevronLeft } from 'lucide-vue-next'
import TablesList from '@/elements/TablesList.vue'
import DataTable from '@/elements/DataTable.vue'
import { Button } from '@/components/ui/button'
import { useOrganizationStore } from '@/stores/organization.store'
import { useBranchStore } from '@/stores/branch.store'
import type { TableRef } from '@/types/models/tableRef'

const route = useRoute()
const organizationStore = useOrganizationStore()
const branchStore = useBranchStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)
const organizationId = computed(() => organizationStore.selectedOrganizationId)

const branch = computed(() => branchStore.branches.find((b) => b.id === branchId.value))
const endpointStatus = computed(() => branch.value?.endpoint_status)
const isRunning = computed(() => endpointStatus.value === 'running')
const isTransient = computed(
  () => endpointStatus.value === 'starting' || endpointStatus.value === 'stopping',
)

const isDesktop = useMediaQuery('(min-width: 768px)')
const selected = ref<TableRef | null>(null)

async function startEndpoint() {
  if (!organizationId.value) return
  await branchStore.launchEndpoint(organizationId.value, projectId.value, branchId.value)
}
</script>

<template>
  <template v-if="isRunning">
    <div v-if="isDesktop" class="grid grid-cols-[260px_1fr] gap-4 h-full">
      <div class="border rounded-lg overflow-hidden flex flex-col">
        <TablesList
          :organization-id="organizationId"
          :project-id="projectId"
          :branch-id="branchId"
          :selected="selected"
          @update:selected="selected = $event"
        />
      </div>
      <div class="border rounded-lg overflow-hidden flex flex-col">
        <div
          v-if="!selected"
          class="flex-1 flex items-center justify-center text-sm text-muted-foreground"
        >
          Select a table to view data
        </div>
        <DataTable
          v-else
          :key="`${branchId}:${selected.schema}.${selected.name}`"
          :organization-id="organizationId"
          :project-id="projectId"
          :branch-id="branchId"
          :schema="selected.schema"
          :table="selected.name"
        />
      </div>
    </div>

    <div v-else class="h-full">
      <div
        v-if="!selected"
        class="h-full border rounded-lg overflow-hidden flex flex-col"
      >
        <TablesList
          :organization-id="organizationId"
          :project-id="projectId"
          :branch-id="branchId"
          :selected="selected"
          @update:selected="selected = $event"
        />
      </div>
      <div v-else class="h-full border rounded-lg overflow-hidden flex flex-col">
        <div class="shrink-0 border-b px-2 py-2 flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            class="cursor-pointer"
            @click="selected = null"
          >
            <ChevronLeft class="size-4" />
            Tables
          </Button>
          <span class="text-sm font-medium truncate">
            <span v-if="selected.schema !== 'public'" class="text-muted-foreground">
              {{ selected.schema }}.
            </span>
            {{ selected.name }}
          </span>
        </div>
        <DataTable
          :key="`${branchId}:${selected.schema}.${selected.name}`"
          :organization-id="organizationId"
          :project-id="projectId"
          :branch-id="branchId"
          :schema="selected.schema"
          :table="selected.name"
        />
      </div>
    </div>
  </template>

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
