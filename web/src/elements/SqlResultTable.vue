<script setup lang="ts">
import { Loader2 } from 'lucide-vue-next'
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
  result: ExecuteSqlResponse | null
  loading: boolean
}>()
</script>

<template>
  <div class="flex flex-col h-full font-mono">
    <div class="shrink-0 border-b px-3 py-2 flex items-center gap-2">
      <span class="text-xs text-muted-foreground">Results</span>
    </div>

    <div v-if="props.loading" class="flex-1 flex items-center justify-center">
      <Loader2 class="size-5 animate-spin text-muted-foreground" />
    </div>

    <div
      v-else-if="!props.result"
      class="flex-1 flex items-center justify-center text-sm text-muted-foreground"
    >
      Run a query to see results
    </div>

    <div
      v-else-if="props.result.error"
      class="flex-1 flex items-start p-4 text-sm text-red-500 font-mono whitespace-pre-wrap"
    >
      {{ props.result.error }}
    </div>

    <div
      v-else-if="props.result.columns.length === 0"
      class="flex-1 flex items-center justify-center text-sm text-muted-foreground"
    >
      Query executed successfully
    </div>

    <div v-else class="flex-1 min-h-0 overflow-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead v-for="column in props.result.columns" :key="column">
              {{ column }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="props.result.rows.length === 0">
            <TableCell :colspan="props.result.columns.length" class="text-center text-muted-foreground py-8">
              No rows
            </TableCell>
          </TableRow>
          <TableRow v-else v-for="(row, rowIndex) in props.result.rows" :key="rowIndex">
            <TableCell v-for="(cell, cellIndex) in row" :key="cellIndex">
              <span v-if="cell === null" class="text-muted-foreground italic text-xs">NULL</span>
              <span v-else>{{ cell }}</span>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  </div>
</template>