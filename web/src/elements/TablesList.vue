<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useSqlStore } from '@/stores/sql.store'
import { Skeleton } from '@/components/ui/skeleton'
import type { TableRef } from '@/types/models/tableRef'

const props = defineProps<{
  organizationId: string
  projectId: string
  branchId: string
  lsn?: string | null
  selected?: TableRef | null
}>()

const emit = defineEmits<{
  'update:selected': [ref: TableRef]
}>()

const sqlStore = useSqlStore()
const tables = ref<TableRef[]>([])

async function load() {
  try {
    tables.value = await sqlStore.listTables(props.organizationId, props.projectId, props.branchId, props.lsn)
  } catch {
    tables.value = []
  }
}

onMounted(load)

watch(() => [props.branchId, props.lsn], load)

function isSelected(tableRef: TableRef): boolean {
  return props.selected?.schema === tableRef.schema && props.selected?.name === tableRef.name
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider border-b">
      Tables
    </div>
    <div class="flex-1 overflow-auto max-h-[calc(100vh-14rem+5px)]">
      <div v-if="sqlStore.tablesLoading" class="flex flex-col gap-2 p-3">
        <Skeleton class="h-6 w-full" />
        <Skeleton class="h-6 w-4/5" />
        <Skeleton class="h-6 w-full" />
        <Skeleton class="h-6 w-3/5" />
      </div>
      <div v-else-if="tables.length === 0" class="px-3 py-6 text-sm text-muted-foreground text-center">
        No tables found
      </div>
      <button
        v-else
        v-for="tableRef in tables"
        :key="`${tableRef.schema}.${tableRef.name}`"
        type="button"
        class="w-full text-left px-3 py-2 text-sm flex items-baseline gap-1 hover:bg-muted/50 cursor-pointer"
        :class="isSelected(tableRef) ? 'bg-muted font-medium' : ''"
        @click="emit('update:selected', tableRef)"
      >
        <span v-if="tableRef.schema !== 'public'" class="text-muted-foreground text-xs">
          {{ tableRef.schema }}.
        </span>
        <span>{{ tableRef.name }}</span>
      </button>
    </div>
  </div>
</template>
