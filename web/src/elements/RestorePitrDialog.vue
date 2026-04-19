<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useDebounceFn, useMediaQuery } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { ChevronDownIcon, Loader2 } from 'lucide-vue-next'
import type { DateValue } from '@internationalized/date'
import { CalendarDate } from '@internationalized/date'
import dayjs, { type Dayjs } from 'dayjs'
import utc from 'dayjs/plugin/utc'
import timezone from 'dayjs/plugin/timezone'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Calendar } from '@/components/ui/calendar'
import TablesList from '@/elements/TablesList.vue'
import DataTable from '@/elements/DataTable.vue'
import { branchesApi } from '@/api/branches'
import { useBranchStore } from '@/stores/branch.store'
import { getAppError } from '@/api/utils'
import type { LsnResponse, LsnKind } from '@/types/dto/lsn'
import type { TableRef } from '@/types/models/tableRef'

dayjs.extend(utc)
dayjs.extend(timezone)

const props = defineProps<{
  open: boolean
  organizationId: string
  projectId: string
  branchId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'restored': []
}>()

const branchStore = useBranchStore()

const selectedAt = ref<Dayjs>(dayjs())
const calendarOpen = ref(false)
const lsn = ref<string | null>(null)
const lsnKind = ref<LsnKind | null>(null)
const lsnLoading = ref(false)
const lsnError = ref<string | null>(null)
const restoring = ref(false)
const selectedTable = ref<TableRef | null>(null)

const isDesktop = useMediaQuery('(min-width: 768px)')

const calendarValue = computed<DateValue>(() =>
  new CalendarDate(selectedAt.value.year(), selectedAt.value.month() + 1, selectedAt.value.date()),
)

const timeValue = computed<string>({
  get: () => selectedAt.value.format('HH:mm:ss'),
  set: (value: string) => {
    const [hours, minutes, seconds] = value.split(':').map(Number)
    if (Number.isNaN(hours) || Number.isNaN(minutes)) return
    selectedAt.value = selectedAt.value
      .hour(hours)
      .minute(minutes)
      .second(Number.isNaN(seconds) ? 0 : seconds)
  },
})

const timezoneLabel = computed<string>(() =>
  `${dayjs.tz.guess()}, GMT${selectedAt.value.format('Z')}`,
)

const canRestore = computed<boolean>(() => {
  return (
    !!lsn.value
    && (lsnKind.value === 'present' || lsnKind.value === 'future' || lsnKind.value === 'past')
    && !restoring.value
    && !lsnLoading.value
  )
})

function onCalendarChange(value: DateValue | undefined) {
  if (!value) return
  selectedAt.value = selectedAt.value
    .year(value.year)
    .month(value.month - 1)
    .date(value.day)
  calendarOpen.value = false
}

async function fetchLsn() {
  lsnLoading.value = true
  lsnError.value = null
  try {
    const response: LsnResponse = await branchesApi.lsn(
      props.organizationId,
      props.projectId,
      props.branchId,
      selectedAt.value.toDate(),
    )
    lsn.value = response.lsn
    lsnKind.value = response.kind
  } catch (error) {
    lsn.value = null
    lsnKind.value = null
    lsnError.value = getAppError(error)
  } finally {
    lsnLoading.value = false
  }
}

const debouncedFetchLsn = useDebounceFn(fetchLsn, 300)

watch(selectedAt, () => {
  debouncedFetchLsn()
})

watch(
  () => props.open,
  (value) => {
    if (value) {
      selectedAt.value = dayjs()
      lsn.value = null
      lsnKind.value = null
      lsnError.value = null
      selectedTable.value = null
      fetchLsn()
    }
  },
)

function selectTable(tableRef: TableRef) {
  selectedTable.value = tableRef
}

async function onRestore() {
  if (!canRestore.value || !lsn.value) return
  restoring.value = true
  try {
    await branchStore.restore(props.organizationId, props.projectId, props.branchId, lsn.value)
    emit('restored')
    emit('update:open', false)
  } catch (error) {
    toast.error(getAppError(error))
  } finally {
    restoring.value = false
  }
}

function onCancel() {
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-5xl">
      <DialogHeader>
        <DialogTitle>Restore from PITR</DialogTitle>
      </DialogHeader>

      <div class="flex flex-col gap-4">
        <div class="flex flex-wrap items-end gap-6">
          <div class="flex flex-col gap-2">
            <Label for="pitr-date" class="px-1">Point in time</Label>
            <div class="flex gap-2">
              <Popover v-model:open="calendarOpen">
                <PopoverTrigger as-child>
                  <Button
                    id="pitr-date"
                    variant="outline"
                    class="w-40 justify-between font-normal cursor-pointer"
                  >
                    {{ selectedAt.toDate().toLocaleDateString() }}
                    <ChevronDownIcon class="size-4 opacity-60" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent class="w-auto overflow-hidden p-0" align="start">
                  <Calendar
                    :model-value="calendarValue"
                    @update:model-value="onCalendarChange"
                  />
                </PopoverContent>
              </Popover>
              <Input
                v-model="timeValue"
                type="time"
                step="1"
                class="w-36 bg-background appearance-none [&::-webkit-calendar-picker-indicator]:hidden [&::-webkit-calendar-picker-indicator]:appearance-none"
              />
            </div>
            <span class="text-xs text-muted-foreground px-1">{{ timezoneLabel }}</span>
          </div>

          <div class="flex flex-col gap-2">
            <Label class="px-1">Resolved LSN</Label>
            <div class="flex items-center gap-2 h-9 px-1">
              <Loader2 v-if="lsnLoading" class="size-4 animate-spin text-muted-foreground" />
              <template v-else-if="lsn">
                <span class="font-mono text-sm">{{ lsn }}</span>
                <span
                  class="text-xs px-2 py-0.5 rounded"
                  :class="lsnKind === 'nodata'
                    ? 'bg-red-100 text-red-700 dark:bg-red-950/50 dark:text-red-300'
                    : 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300'"
                >{{ lsnKind }}</span>
              </template>
              <span v-else-if="lsnError" class="text-xs text-red-600">{{ lsnError }}</span>
              <span v-else class="text-xs text-muted-foreground">—</span>
            </div>
          </div>
        </div>

        <div v-if="lsn" class="border rounded-lg overflow-hidden h-105">
          <div v-if="isDesktop" class="grid grid-cols-[260px_1fr] h-full">
            <div class="border-r flex flex-col overflow-hidden">
              <TablesList
                :organization-id="organizationId"
                :project-id="projectId"
                :branch-id="branchId"
                :lsn="lsn"
                :selected="selectedTable"
                @update:selected="selectTable($event)"
              />
            </div>
            <div class="flex flex-col overflow-hidden">
              <div
                v-if="!selectedTable"
                class="flex-1 flex items-center justify-center text-sm text-muted-foreground"
              >
                Select a table to preview data at this LSN
              </div>
              <DataTable
                v-else
                :key="`${branchId}:${selectedTable.schema}.${selectedTable.name}:${lsn}`"
                :organization-id="organizationId"
                :project-id="projectId"
                :branch-id="branchId"
                :schema="selectedTable.schema"
                :table="selectedTable.name"
                :lsn="lsn"
              />
            </div>
          </div>
          <div v-else class="flex items-center justify-center h-full text-sm text-muted-foreground px-6 text-center">
            Preview is available on wider screens
          </div>
        </div>
      </div>

      <DialogFooter class="mt-2">
        <Button variant="outline" class="cursor-pointer" :disabled="restoring" @click="onCancel">
          Cancel
        </Button>
        <Button
          variant="destructive"
          class="cursor-pointer"
          :disabled="!canRestore"
          @click="onRestore"
        >
          <Loader2 v-if="restoring" class="mr-1.5 size-3.5 animate-spin" />
          Restore
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>