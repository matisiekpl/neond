<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { useSqlStore } from '@/stores/sql.store'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import type { ExecuteSqlResponse } from '@/types/dto/executeSqlResponse'

const props = defineProps<{
  organizationId: string
  projectId: string
  branchId: string
  schema: string
  table: string
  lsn?: string | null
}>()

const PAGE_SIZE = 50

const sqlStore = useSqlStore()
const response = ref<ExecuteSqlResponse | null>(null)
const page = ref(1)

const rows = computed(() => response.value?.rows.slice(0, PAGE_SIZE) ?? [])
const hasNextPage = computed(() => (response.value?.rows.length ?? 0) > PAGE_SIZE)
const startRow = computed(() => rows.value.length === 0 ? 0 : (page.value - 1) * PAGE_SIZE + 1)
const endRow = computed(() => (page.value - 1) * PAGE_SIZE + rows.value.length)
const canPrev = computed(() => page.value > 1 && !sqlStore.rowsLoading)
const canNext = computed(() => hasNextPage.value && !sqlStore.rowsLoading)

async function load() {
  try {
    response.value = await sqlStore.fetchTableData(
      props.organizationId,
      props.projectId,
      props.branchId,
      { schema: props.schema, name: props.table },
      page.value,
      PAGE_SIZE,
      props.lsn,
    )
  } catch {
    response.value = null
  }
}

async function reset() {
  page.value = 1
  await load()
}

onMounted(reset)

watch(() => [props.branchId, props.schema, props.table, props.lsn], reset)
watch(page, load)

function prev() {
  if (canPrev.value) page.value--
}

function next() {
  if (canNext.value) page.value++
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div v-if="sqlStore.rowsLoading" class="flex flex-col gap-2 p-4">
      <Skeleton class="h-8 w-full" />
      <Skeleton class="h-6 w-full" />
      <Skeleton class="h-6 w-full" />
      <Skeleton class="h-6 w-4/5" />
    </div>
    <template v-else-if="response">
      <div class="flex-1 min-h-0 [&>[data-slot=table-container]]:h-full">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead v-for="column in response.columns" :key="column">
                {{ column }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-if="rows.length === 0">
              <TableCell :colspan="response.columns.length" class="text-center text-muted-foreground py-8">
                No rows
              </TableCell>
            </TableRow>
            <TableRow v-else v-for="(row, rowIndex) in rows" :key="rowIndex">
              <TableCell v-for="(cell, cellIndex) in row" :key="cellIndex">
                <span v-if="cell === null" class="text-muted-foreground italic text-xs">NULL</span>
                <span v-else>{{ cell }}</span>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
      <div class="border-t px-4 py-2 text-xs text-muted-foreground shrink-0 flex items-center justify-between gap-2">
        <span>Showing {{ startRow }}–{{ endRow }}</span>
        <div class="flex items-center gap-2">
          <span>Page {{ page }}</span>
          <Button variant="outline" size="sm" :disabled="!canPrev" class="cursor-pointer" @click="prev">
            <ChevronLeft class="size-4" />
          </Button>
          <Button variant="outline" size="sm" :disabled="!canNext" class="cursor-pointer" @click="next">
            <ChevronRight class="size-4" />
          </Button>
        </div>
      </div>
    </template>
  </div>
</template>
