<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import {
  ChevronLeft,
  ChevronRight,
  ArrowUp,
  ArrowDown,
  Filter,
  MoreHorizontal,
  X,
  Plus,
  Loader2,
} from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useSqlStore } from '@/stores/sql.store'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import type { ExecuteSqlResponse } from '@/types/dto/executeSqlResponse'
import type { TableFilter, TableSort, FilterOperator, SortDirection } from '@/types/dto/tableFilter'
import type { RowUpdate } from '@/types/dto/rowUpdate'
import { toCsv } from '@/lib/csv'
import { downloadBlob } from '@/lib/download'

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

const filters = ref<TableFilter[]>([])
const filterColumnPopoverOpen = ref<boolean[]>([])
const showFilters = ref(false)
const sort = ref<TableSort | null>(null)
const selectedRows = ref<Set<number>>(new Set())

const primaryKeyColumns = ref<string[]>([])
const edits = ref<Map<string, string>>(new Map())
const failedRows = ref<Map<number, string>>(new Map())
const newRows = ref<Record<string, string>[]>([])
const failedNewRows = ref<Map<number, string>>(new Map())
const pendingDeletes = ref<Set<number>>(new Set())
const failedDeletes = ref<Map<number, string>>(new Map())
const savingEdits = ref(false)

const rows = computed(() => response.value?.rows.slice(0, PAGE_SIZE) ?? [])
const hasNextPage = computed(() => (response.value?.rows.length ?? 0) > PAGE_SIZE)
const startRow = computed(() => rows.value.length === 0 ? 0 : (page.value - 1) * PAGE_SIZE + 1)
const endRow = computed(() => (page.value - 1) * PAGE_SIZE + rows.value.length)
const canPrev = computed(() => page.value > 1 && !sqlStore.rowsLoading)
const canNext = computed(() => hasNextPage.value && !sqlStore.rowsLoading)

const canEdit = computed(() => primaryKeyColumns.value.length > 0 && !props.lsn)

const allSelected = computed(() => rows.value.length > 0 && selectedRows.value.size === rows.value.length)
const someSelected = computed(() => selectedRows.value.size > 0 && !allSelected.value)

const FILTER_OPERATORS: { value: FilterOperator; label: string }[] = [
  { value: 'equals', label: 'equals' },
  { value: 'not_equals', label: 'not equals' },
  { value: 'contains', label: 'contains' },
  { value: 'starts_with', label: 'starts with' },
  { value: 'greater_than', label: 'greater than' },
  { value: 'less_than', label: 'less than' },
  { value: 'is_null', label: 'is null' },
  { value: 'is_not_null', label: 'is not null' },
]

async function load() {
  try {
    response.value = await sqlStore.fetchTableData(
      props.organizationId,
      props.projectId,
      props.branchId,
      { schema: props.schema, name: props.table },
      page.value,
      PAGE_SIZE,
      filters.value,
      sort.value,
      primaryKeyColumns.value,
      props.lsn,
    )
  } catch {
    response.value = null
  }
}

async function reset() {
  page.value = 1
  filters.value = []
  sort.value = null
  selectedRows.value = new Set()
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
  try {
    primaryKeyColumns.value = await sqlStore.fetchPrimaryKey(
      props.organizationId,
      props.projectId,
      props.branchId,
      { schema: props.schema, name: props.table },
      props.lsn,
    )
  } catch {
    primaryKeyColumns.value = []
  }
  await load()
}

const debouncedLoad = useDebounceFn(load, 300)

onMounted(reset)

watch(() => [props.branchId, props.schema, props.table, props.lsn], reset)
watch(page, () => {
  selectedRows.value = new Set()
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
  load()
})

function prev() {
  if (canPrev.value) page.value--
}

function next() {
  if (canNext.value) page.value++
}

function toggleSort(column: string) {
  if (sort.value?.column !== column) {
    sort.value = { column, direction: 'asc' }
  } else if (sort.value.direction === 'asc') {
    sort.value = { column, direction: 'desc' }
  } else {
    sort.value = null
  }
  page.value = 1
  selectedRows.value = new Set()
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
  load()
}

function getSortDirection(column: string): SortDirection | null {
  return sort.value?.column === column ? sort.value.direction : null
}

function addFilter() {
  const firstColumn = response.value?.columns[0] ?? ''
  filters.value.push({ column: firstColumn, operator: 'equals', value: '' })
  filterColumnPopoverOpen.value.push(false)
}

function removeFilter(index: number) {
  filters.value.splice(index, 1)
  filterColumnPopoverOpen.value.splice(index, 1)
  page.value = 1
  selectedRows.value = new Set()
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
  load()
}

function clearFilters() {
  filters.value = []
  page.value = 1
  selectedRows.value = new Set()
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
  load()
}

function onFilterColumnChange(index: number, column: string) {
  filters.value[index].column = column
  page.value = 1
  selectedRows.value = new Set()
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
  load()
}

function onFilterOperatorChange(index: number, operator: FilterOperator) {
  filters.value[index].operator = operator
  page.value = 1
  selectedRows.value = new Set()
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
  load()
}

function onFilterValueInput() {
  page.value = 1
  selectedRows.value = new Set()
  edits.value = new Map()
  debouncedLoad()
}

function toggleAllRows(value: boolean | 'indeterminate') {
  if (value === true) {
    selectedRows.value = new Set(rows.value.map((_, index) => index))
  } else {
    selectedRows.value = new Set()
  }
}

function onCellBlur(event: FocusEvent, rowIndex: number, columnIndex: number, originalValue: string | null) {
  const target = event.target as HTMLElement
  const text = target.textContent ?? ''
  const key = `${rowIndex}:${columnIndex}`
  if (text === (originalValue ?? '')) {
    edits.value.delete(key)
  } else {
    edits.value.set(key, text)
  }
  edits.value = new Map(edits.value)
  if (failedRows.value.has(rowIndex)) {
    failedRows.value.delete(rowIndex)
    failedRows.value = new Map(failedRows.value)
  }
}

function onCellKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    event.preventDefault()
    ;(event.target as HTMLElement).blur()
  } else if (event.key === 'Escape') {
    event.preventDefault()
    ;(event.target as HTMLElement).blur()
  }
}

function rejectChanges() {
  edits.value = new Map()
  failedRows.value = new Map()
  newRows.value = []
  failedNewRows.value = new Map()
  pendingDeletes.value = new Set()
  failedDeletes.value = new Map()
}

function deleteSelectedRows() {
  if (selectedRows.value.size === 0) return
  const next = new Set(pendingDeletes.value)
  for (const rowIndex of selectedRows.value) next.add(rowIndex)
  pendingDeletes.value = next
  selectedRows.value = new Set()
}

function undoPendingDelete(rowIndex: number) {
  const next = new Set(pendingDeletes.value)
  next.delete(rowIndex)
  pendingDeletes.value = next
  if (failedDeletes.value.has(rowIndex)) {
    failedDeletes.value.delete(rowIndex)
    failedDeletes.value = new Map(failedDeletes.value)
  }
}

function addNewRow() {
  newRows.value = [...newRows.value, {}]
}

function removeNewRow(index: number) {
  newRows.value = newRows.value.filter((_, i) => i !== index)
  if (failedNewRows.value.has(index)) {
    const next = new Map<number, string>()
    for (const [key, error] of failedNewRows.value) {
      if (key < index) next.set(key, error)
      else if (key > index) next.set(key - 1, error)
    }
    failedNewRows.value = next
  }
}

function onNewRowCellBlur(event: FocusEvent, newRowIndex: number, columnName: string) {
  const text = (event.target as HTMLElement).textContent ?? ''
  const row = { ...newRows.value[newRowIndex], [columnName]: text }
  const next = [...newRows.value]
  next[newRowIndex] = row
  newRows.value = next
  if (failedNewRows.value.has(newRowIndex)) {
    failedNewRows.value.delete(newRowIndex)
    failedNewRows.value = new Map(failedNewRows.value)
  }
}

async function acceptChanges() {
  if (edits.value.size === 0 && newRows.value.length === 0 && pendingDeletes.value.size === 0) return
  const columns = response.value?.columns ?? []
  const rowsByIndex = new Map<number, RowUpdate>()
  for (const [key, newValue] of edits.value) {
    const [rowIndexString, columnIndexString] = key.split(':')
    const rowIndex = Number(rowIndexString)
    const columnIndex = Number(columnIndexString)
    const columnName = columns[columnIndex]
    if (!columnName) continue
    let update = rowsByIndex.get(rowIndex)
    if (!update) {
      const primaryKeyValues: Record<string, string | null> = {}
      for (const primaryKeyColumn of primaryKeyColumns.value) {
        const primaryKeyColumnIndex = columns.indexOf(primaryKeyColumn)
        if (primaryKeyColumnIndex === -1) continue
        primaryKeyValues[primaryKeyColumn] = rows.value[rowIndex][primaryKeyColumnIndex]
      }
      update = { primaryKeyValues, changedCells: {} }
      rowsByIndex.set(rowIndex, update)
    }
    update.changedCells[columnName] = newValue
  }
  const rowIndices = Array.from(rowsByIndex.keys())
  const updates = Array.from(rowsByIndex.values())
  savingEdits.value = true
  try {
    let failedCount = 0
    if (updates.length > 0) {
      const results = await sqlStore.updateRows(
        props.organizationId,
        props.projectId,
        props.branchId,
        { schema: props.schema, name: props.table },
        updates,
        props.lsn,
      )
      const nextEdits = new Map(edits.value)
      const nextFailed = new Map<number, string>()
      results.forEach((result, index) => {
        const rowIndex = rowIndices[index]
        if (result.error) {
          failedCount++
          nextFailed.set(rowIndex, result.error)
        } else {
          for (const key of Array.from(nextEdits.keys())) {
            if (key.startsWith(`${rowIndex}:`)) nextEdits.delete(key)
          }
        }
      })
      edits.value = nextEdits
      failedRows.value = nextFailed
    }
    if (newRows.value.length > 0) {
      const insertResults = await sqlStore.insertRows(
        props.organizationId,
        props.projectId,
        props.branchId,
        { schema: props.schema, name: props.table },
        newRows.value,
        props.lsn,
      )
      const remainingRows: Record<string, string>[] = []
      const nextFailedNew = new Map<number, string>()
      insertResults.forEach((result, index) => {
        if (result.error) {
          failedCount++
          nextFailedNew.set(remainingRows.length, result.error)
          remainingRows.push(newRows.value[index])
        }
      })
      newRows.value = remainingRows
      failedNewRows.value = nextFailedNew
    }
    if (pendingDeletes.value.size > 0) {
      const columns = response.value?.columns ?? []
      const deleteIndices = Array.from(pendingDeletes.value)
      const primaryKeyValuesList = deleteIndices.map((rowIndex) => {
        const primaryKeyValues: Record<string, string | null> = {}
        for (const primaryKeyColumn of primaryKeyColumns.value) {
          const primaryKeyColumnIndex = columns.indexOf(primaryKeyColumn)
          if (primaryKeyColumnIndex === -1) continue
          primaryKeyValues[primaryKeyColumn] = rows.value[rowIndex][primaryKeyColumnIndex]
        }
        return primaryKeyValues
      })
      const deleteResults = await sqlStore.deleteRows(
        props.organizationId,
        props.projectId,
        props.branchId,
        { schema: props.schema, name: props.table },
        primaryKeyValuesList,
        props.lsn,
      )
      const nextPendingDeletes = new Set<number>()
      const nextFailedDeletes = new Map<number, string>()
      deleteResults.forEach((result, index) => {
        const rowIndex = deleteIndices[index]
        if (result.error) {
          failedCount++
          nextPendingDeletes.add(rowIndex)
          nextFailedDeletes.set(rowIndex, result.error)
        }
      })
      pendingDeletes.value = nextPendingDeletes
      failedDeletes.value = nextFailedDeletes
    }
    if (failedCount === 0) {
      toast.success('Changes saved')
      await load()
    } else {
      toast.error(`${failedCount} row${failedCount === 1 ? '' : 's'} failed`)
    }
  } finally {
    savingEdits.value = false
  }
}

function toggleRow(index: number, value: boolean | 'indeterminate') {
  const next = new Set(selectedRows.value)
  if (value === true) {
    next.add(index)
  } else {
    next.delete(index)
  }
  selectedRows.value = next
}

function rowsToJsonObjects(columns: string[], targetRows: (string | null)[][]): object[] {
  return targetRows.map((row) => Object.fromEntries(columns.map((column, index) => [column, row[index]])))
}

function exportSelected(format: 'csv' | 'json') {
  const columns = response.value?.columns ?? []
  const selectedRowsData = Array.from(selectedRows.value)
    .sort((a, b) => a - b)
    .map((index) => rows.value[index])
  const filename = `${props.schema}.${props.table}.selected`
  if (format === 'csv') {
    const content = toCsv(columns, selectedRowsData)
    downloadBlob(`${filename}.csv`, new Blob([content], { type: 'text/csv' }))
  } else {
    const content = JSON.stringify(rowsToJsonObjects(columns, selectedRowsData), null, 2)
    downloadBlob(`${filename}.json`, new Blob([content], { type: 'application/json' }))
  }
}

function exportAll(format: 'csv' | 'json') {
  const filename = `${props.schema}.${props.table}`
  sqlStore.fetchAllTableData(
    props.organizationId,
    props.projectId,
    props.branchId,
    { schema: props.schema, name: props.table },
    filters.value,
    sort.value,
    primaryKeyColumns.value,
    props.lsn,
  ).then((allData: ExecuteSqlResponse) => {
    if (format === 'csv') {
      const content = toCsv(allData.columns, allData.rows)
      downloadBlob(`${filename}.csv`, new Blob([content], { type: 'text/csv' }))
    } else {
      const content = JSON.stringify(rowsToJsonObjects(allData.columns, allData.rows), null, 2)
      downloadBlob(`${filename}.json`, new Blob([content], { type: 'application/json' }))
    }
  })
}
</script>

<template>
  <div class="flex flex-col h-full font-mono">
    <div class="border-b px-3 py-2 flex items-center gap-2 shrink-0 overflow-x-auto [&>*]:shrink-0">
      <Button
        variant="outline"
        size="sm"
        class="cursor-pointer gap-1.5"
        :class="{ 'bg-accent': showFilters }"
        @click="showFilters = !showFilters"
      >
        <Filter class="size-3.5" />
        Filters
        <span v-if="filters.length > 0" class="text-xs font-medium">({{ filters.length }})</span>
      </Button>

      <Button
        v-if="canEdit"
        variant="outline"
        size="sm"
        class="cursor-pointer gap-1.5"
        @click="addNewRow"
      >
        <Plus class="size-3.5" />
        Add row
      </Button>

      <Button
        v-if="canEdit && selectedRows.size > 0"
        variant="outline"
        size="sm"
        class="cursor-pointer gap-1.5 text-red-600 hover:text-red-700"
        @click="deleteSelectedRows"
      >
        <X class="size-3.5" />
        Delete {{ selectedRows.size }}
      </Button>

      <div class="flex-1" />

      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button variant="outline" size="sm" class="cursor-pointer">
            <MoreHorizontal class="size-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem class="cursor-pointer" @click="reset">Refresh rows</DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem class="cursor-pointer" @click="exportAll('json')">Export all to .json</DropdownMenuItem>
          <DropdownMenuItem class="cursor-pointer" @click="exportAll('csv')">Export all to .csv</DropdownMenuItem>
          <template v-if="selectedRows.size > 0">
            <DropdownMenuSeparator />
            <DropdownMenuItem class="cursor-pointer" @click="exportSelected('json')">Export selected to .json</DropdownMenuItem>
            <DropdownMenuItem class="cursor-pointer" @click="exportSelected('csv')">Export selected to .csv</DropdownMenuItem>
          </template>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    <div class="h-[calc(100vh-16rem)] md:h-[calc(100vh-15rem-5px)] flex flex-col">
      <div v-if="showFilters" class="border-b px-3 py-2 flex flex-col gap-2 shrink-0">
        <div
          v-for="(filter, index) in filters"
          :key="index"
          class="flex items-center gap-2"
        >
          <Button variant="ghost" size="sm" class="cursor-pointer px-1.5" @click="removeFilter(index)">
            <X class="size-3.5" />
          </Button>
          <span class="text-xs text-muted-foreground w-8 shrink-0 mr-3">{{ index === 0 ? 'where' : 'and' }}</span>

          <Popover v-model:open="filterColumnPopoverOpen[index]">
            <PopoverTrigger as-child>
              <Button variant="outline" size="sm" class="cursor-pointer min-w-30 justify-between">
                {{ filter.column || 'column' }}
                <ChevronRight class="size-3 ml-1 opacity-50 rotate-90" />
              </Button>
            </PopoverTrigger>
            <PopoverContent class="w-48 p-0">
              <Command>
                <CommandInput placeholder="Search column..." class="h-8 text-xs" />
                <CommandList>
                  <CommandEmpty>No column found.</CommandEmpty>
                  <CommandGroup>
                    <CommandItem
                      v-for="column in response?.columns ?? []"
                      :key="column"
                      :value="column"
                      class="cursor-pointer text-xs"
                      @select="() => { onFilterColumnChange(index, column); filterColumnPopoverOpen[index] = false }"
                    >
                      {{ column }}
                    </CommandItem>
                  </CommandGroup>
                </CommandList>
              </Command>
            </PopoverContent>
          </Popover>

          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="outline" size="sm" class="cursor-pointer min-w-[110px] justify-between">
                {{ FILTER_OPERATORS.find((op) => op.value === filter.operator)?.label }}
                <ChevronRight class="size-3 ml-1 opacity-50 rotate-90" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem
                v-for="op in FILTER_OPERATORS"
                :key="op.value"
                class="cursor-pointer"
                @click="onFilterOperatorChange(index, op.value)"
              >
                {{ op.label }}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <Input
            v-if="filter.operator !== 'is_null' && filter.operator !== 'is_not_null'"
            v-model="filter.value"
            size="sm"
            class="h-8 text-xs max-w-[200px]"
            placeholder="value"
            @input="onFilterValueInput"
          />
        </div>

        <div class="flex items-center gap-2">
          <Button variant="ghost" size="sm" class="cursor-pointer gap-1 text-xs" @click="addFilter">
            <Plus class="size-3.5" />
            Add filter
          </Button>
          <Button
            v-if="filters.length > 0"
            variant="ghost"
            size="sm"
            class="cursor-pointer text-xs text-muted-foreground"
            @click="clearFilters"
          >
            Clear filters
          </Button>
        </div>
      </div>

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
              <TableHead class="w-10 px-3">
                <Checkbox
                  :model-value="allSelected ? true : someSelected ? 'indeterminate' : false"
                  @update:model-value="toggleAllRows"
                />
              </TableHead>
              <TableHead
                v-for="column in response.columns"
                :key="column"
                class="cursor-pointer select-none whitespace-nowrap"
                @click="toggleSort(column)"
              >
                <div class="flex items-center gap-1">
                  {{ column }}
                  <ArrowUp v-if="getSortDirection(column) === 'asc'" class="size-3 shrink-0" />
                  <ArrowDown v-else-if="getSortDirection(column) === 'desc'" class="size-3 shrink-0" />
                </div>
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody class="overflow-y-scroll">
            <TableRow
              v-for="(newRow, newRowIndex) in newRows"
              :key="`new-${newRowIndex}`"
              :class="failedNewRows.has(newRowIndex)
                ? 'bg-red-100/60 dark:bg-red-900/30'
                : 'bg-emerald-50 dark:bg-emerald-950/30'"
              :title="failedNewRows.get(newRowIndex)"
            >
              <TableCell class="px-3">
                <button
                  type="button"
                  class="cursor-pointer text-muted-foreground hover:text-foreground disabled:opacity-50"
                  :disabled="savingEdits"
                  @click="removeNewRow(newRowIndex)"
                >
                  <X class="size-3.5" />
                </button>
              </TableCell>
              <TableCell
                v-for="(column, columnIndex) in response.columns"
                :key="columnIndex"
              >
                <span
                  class="outline-none block min-w-4"
                  :contenteditable="!savingEdits"
                  spellcheck="false"
                  @blur="onNewRowCellBlur($event, newRowIndex, column)"
                  @keydown="onCellKeydown"
                >{{ newRow[column] ?? '' }}</span>
              </TableCell>
            </TableRow>
            <TableRow v-if="rows.length === 0 && newRows.length === 0">
              <TableCell :colspan="response.columns.length + 1" class="text-center text-muted-foreground py-8">
                No rows
              </TableCell>
            </TableRow>
            <TableRow
              v-for="(row, rowIndex) in rows"
              :key="rowIndex"
              :class="[
                failedDeletes.has(rowIndex) ? 'bg-red-100/60 dark:bg-red-900/30' : '',
                pendingDeletes.has(rowIndex) && !failedDeletes.has(rowIndex) ? 'opacity-50 line-through' : '',
                !pendingDeletes.has(rowIndex) && selectedRows.has(rowIndex) ? 'bg-accent/40' : '',
              ]"
              :title="failedDeletes.get(rowIndex)"
            >
              <TableCell class="px-3">
                <button
                  v-if="pendingDeletes.has(rowIndex)"
                  type="button"
                  class="cursor-pointer text-muted-foreground hover:text-foreground disabled:opacity-50"
                  :disabled="savingEdits"
                  @click="undoPendingDelete(rowIndex)"
                >
                  <X class="size-3.5" />
                </button>
                <Checkbox
                  v-else
                  :model-value="selectedRows.has(rowIndex)"
                  @update:model-value="(value: boolean | 'indeterminate') => toggleRow(rowIndex, value)"
                />
              </TableCell>
              <TableCell
                v-for="(cell, cellIndex) in row"
                :key="cellIndex"
                :class="{
                  'bg-amber-100 dark:bg-amber-900/40': edits.has(`${rowIndex}:${cellIndex}`) && !failedRows.has(rowIndex),
                  'bg-red-100 dark:bg-red-900/40': edits.has(`${rowIndex}:${cellIndex}`) && failedRows.has(rowIndex),
                }"
                :title="edits.has(`${rowIndex}:${cellIndex}`) ? failedRows.get(rowIndex) : undefined"
              >
                <span
                  v-if="canEdit && !primaryKeyColumns.includes(response.columns[cellIndex]) && !pendingDeletes.has(rowIndex)"
                  class="outline-none block"
                  :contenteditable="!savingEdits"
                  spellcheck="false"
                  @blur="onCellBlur($event, rowIndex, cellIndex, cell)"
                  @keydown="onCellKeydown"
                >{{ edits.get(`${rowIndex}:${cellIndex}`) ?? cell ?? '' }}</span>
                <template v-else>
                  <span v-if="cell === null" class="text-muted-foreground italic text-xs">NULL</span>
                  <span v-else>{{ cell }}</span>
                </template>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
      <div
        v-if="edits.size > 0 || newRows.length > 0 || pendingDeletes.size > 0"
        class="border-t px-4 py-2 text-xs shrink-0 flex items-center justify-between gap-2"
        :class="failedRows.size > 0 || failedNewRows.size > 0 || failedDeletes.size > 0
          ? 'bg-red-50 dark:bg-red-950/40'
          : 'bg-amber-50 dark:bg-amber-950/40'"
      >
        <span>
          <span v-if="edits.size > 0">{{ edits.size }} unsaved {{ edits.size === 1 ? 'change' : 'changes' }}</span>
          <span v-if="edits.size > 0 && newRows.length > 0"> · </span>
          <span v-if="newRows.length > 0">{{ newRows.length }} new {{ newRows.length === 1 ? 'row' : 'rows' }}</span>
          <span v-if="(edits.size > 0 || newRows.length > 0) && pendingDeletes.size > 0"> · </span>
          <span v-if="pendingDeletes.size > 0">{{ pendingDeletes.size }} to delete</span>
          <span v-if="failedRows.size + failedNewRows.size + failedDeletes.size > 0"> · {{ failedRows.size + failedNewRows.size + failedDeletes.size }} failed</span>
        </span>
        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" class="cursor-pointer" :disabled="savingEdits" @click="rejectChanges">
            Reject
          </Button>
          <Button size="sm" class="cursor-pointer" :disabled="savingEdits" @click="acceptChanges">
            <Loader2 v-if="savingEdits" class="mr-1.5 size-3.5 animate-spin" />
            Accept
          </Button>
        </div>
      </div>
      <div class="border-t px-4 py-2 text-xs text-muted-foreground shrink-0 flex items-center justify-between gap-2">
        <span>
          Showing {{ startRow }}–{{ endRow }}
          <span v-if="selectedRows.size > 0"> · {{ selectedRows.size }} selected</span>
        </span>
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
  </div>
</template>
