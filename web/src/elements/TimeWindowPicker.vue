<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ChevronDownIcon } from 'lucide-vue-next'
import type { DateValue } from '@internationalized/date'
import { CalendarDate } from '@internationalized/date'
import dayjs, { type Dayjs } from 'dayjs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Calendar } from '@/components/ui/calendar'
import { useMetricStore, type MetricRange, rangeDurationMs, RANGE_PRESETS } from '@/stores/metric.store'

const metricStore = useMetricStore()

const open = ref(false)
const fromDayjs = ref<Dayjs>(dayjs(metricStore.rangeStart))
const toDayjs = ref<Dayjs>(dayjs(metricStore.rangeEnd))
const fromCalendarOpen = ref(false)
const toCalendarOpen = ref(false)

const triggerLabel = computed<string>(() => {
  const preset = RANGE_PRESETS.find((option) => option.value === metricStore.range)
  if (preset) return preset.label
  const from = dayjs(metricStore.rangeStart).format('MMM D, HH:mm')
  const to = dayjs(metricStore.rangeEnd).format('MMM D, HH:mm')
  return `${from} – ${to}`
})

const fromCalendarValue = computed<DateValue>(() =>
  new CalendarDate(fromDayjs.value.year(), fromDayjs.value.month() + 1, fromDayjs.value.date()),
)

const toCalendarValue = computed<DateValue>(() =>
  new CalendarDate(toDayjs.value.year(), toDayjs.value.month() + 1, toDayjs.value.date()),
)

const fromTimeValue = computed<string>({
  get: () => fromDayjs.value.format('HH:mm:ss'),
  set: (value: string) => {
    const [hours, minutes, seconds] = value.split(':').map(Number)
    if (Number.isNaN(hours) || Number.isNaN(minutes)) return
    fromDayjs.value = fromDayjs.value
      .hour(hours)
      .minute(minutes)
      .second(Number.isNaN(seconds) ? 0 : seconds)
  },
})

const toTimeValue = computed<string>({
  get: () => toDayjs.value.format('HH:mm:ss'),
  set: (value: string) => {
    const [hours, minutes, seconds] = value.split(':').map(Number)
    if (Number.isNaN(hours) || Number.isNaN(minutes)) return
    toDayjs.value = toDayjs.value
      .hour(hours)
      .minute(minutes)
      .second(Number.isNaN(seconds) ? 0 : seconds)
  },
})

function onFromCalendarChange(value: DateValue | undefined) {
  if (!value) return
  fromDayjs.value = fromDayjs.value.year(value.year).month(value.month - 1).date(value.day)
  fromCalendarOpen.value = false
}

function onToCalendarChange(value: DateValue | undefined) {
  if (!value) return
  toDayjs.value = toDayjs.value.year(value.year).month(value.month - 1).date(value.day)
  toCalendarOpen.value = false
}

function onPresetClick(preset: MetricRange) {
  const to = dayjs()
  fromDayjs.value = to.subtract(rangeDurationMs[preset], 'ms')
  toDayjs.value = to
  metricStore.setRange(preset)
  open.value = false
}

function applyCustom() {
  metricStore.setCustomRange(fromDayjs.value.toDate(), toDayjs.value.toDate())
  open.value = false
}

watch(
  () => [metricStore.rangeStart, metricStore.rangeEnd] as const,
  ([start, end]) => {
    fromDayjs.value = dayjs(start)
    toDayjs.value = dayjs(end)
  },
)
</script>

<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button variant="outline" class="cursor-pointer justify-between gap-2 font-normal">
        <span v-if="metricStore.isLive" class="relative flex size-2">
          <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75" />
          <span class="relative inline-flex size-2 rounded-full bg-emerald-500" />
        </span>
        {{ triggerLabel }}
        <ChevronDownIcon class="size-4 opacity-60" />
      </Button>
    </PopoverTrigger>
    <PopoverContent class="w-64 p-0" align="end">
      <div class="flex flex-col">
        <div class="flex flex-col py-1">
          <button
            v-for="(option, index) in RANGE_PRESETS"
            :key="option.value"
            class="flex w-full cursor-pointer items-center justify-between px-3 py-1.5 text-left text-xs transition-colors hover:bg-accent"
            :class="metricStore.range === option.value ? 'font-medium text-foreground' : 'text-muted-foreground'"
            @click="onPresetClick(option.value)"
          >
            <span class="flex items-center gap-2">
              <kbd class="flex size-4 items-center justify-center rounded border border-border bg-muted font-mono text-[10px] text-muted-foreground">
                {{ index + 1 }}
              </kbd>
              {{ option.label }}
            </span>
            <span
              v-if="metricStore.range === option.value"
              class="size-1.5 rounded-full bg-foreground"
            />
          </button>
        </div>

        <Separator />

        <div class="flex flex-col gap-3 p-3">
          <div class="flex flex-col gap-1.5">
            <Label class="text-xs text-muted-foreground">From</Label>
            <div class="flex gap-1.5">
              <Popover v-model:open="fromCalendarOpen">
                <PopoverTrigger as-child>
                  <Button variant="outline" class="h-8 flex-1 cursor-pointer justify-between px-2 text-xs font-normal">
                    {{ fromDayjs.toDate().toLocaleDateString() }}
                    <ChevronDownIcon class="size-3 opacity-60" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent class="w-auto overflow-hidden p-0" align="start">
                  <Calendar :model-value="fromCalendarValue" @update:model-value="onFromCalendarChange" />
                </PopoverContent>
              </Popover>
              <Input
                v-model="fromTimeValue"
                type="time"
                step="1"
                class="h-8 w-26 appearance-none bg-background text-xs [&::-webkit-calendar-picker-indicator]:hidden"
              />
            </div>
          </div>

          <div class="flex flex-col gap-1.5">
            <Label class="text-xs text-muted-foreground">To</Label>
            <div class="flex gap-1.5">
              <Popover v-model:open="toCalendarOpen">
                <PopoverTrigger as-child>
                  <Button variant="outline" class="h-8 flex-1 cursor-pointer justify-between px-2 text-xs font-normal">
                    {{ toDayjs.toDate().toLocaleDateString() }}
                    <ChevronDownIcon class="size-3 opacity-60" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent class="w-auto overflow-hidden p-0" align="start">
                  <Calendar :model-value="toCalendarValue" @update:model-value="onToCalendarChange" />
                </PopoverContent>
              </Popover>
              <Input
                v-model="toTimeValue"
                type="time"
                step="1"
                class="h-8 w-26 appearance-none bg-background text-xs [&::-webkit-calendar-picker-indicator]:hidden"
              />
            </div>
          </div>
          <Button size="sm" class="cursor-pointer" @click="applyCustom">Apply custom range</Button>
        </div>
      </div>
    </PopoverContent>
  </Popover>
</template>
