<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useMediaQuery } from '@vueuse/core'
import { ChevronLeft } from 'lucide-vue-next'
import TablesList from '@/elements/TablesList.vue'
import DataTable from '@/elements/DataTable.vue'
import EndpointGate from '@/elements/EndpointGate.vue'
import { Button } from '@/components/ui/button'
import { useOrganizationStore } from '@/stores/organization.store'
import type { TableRef } from '@/types/models/tableRef'

const route = useRoute()
const organizationStore = useOrganizationStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)
const organizationId = computed(() => organizationStore.selectedOrganizationId)

const isDesktop = useMediaQuery('(min-width: 768px)')
const selected = ref<TableRef | null>(null)
</script>

<template>
  <EndpointGate
    v-if="organizationId"
    :organization-id="organizationId"
    :project-id="projectId"
    :branch-id="branchId"
  >
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
  </EndpointGate>
</template>