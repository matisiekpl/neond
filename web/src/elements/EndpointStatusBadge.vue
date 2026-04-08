<script setup lang="ts">
import type {BranchStatus} from '@/types/models/branch'

defineProps<{
  status: BranchStatus
}>()

const STATUS_CONFIG: Record<BranchStatus, { label: string; dotClass: string }> = {
  running: {label: 'Running', dotClass: 'bg-green-500'},
  starting: {label: 'Starting', dotClass: 'bg-amber-400'},
  stopping: {label: 'Stopping', dotClass: 'bg-amber-400'},
  stopped: {label: 'Stopped', dotClass: 'bg-muted-foreground'},
  failed: {label: 'Failed', dotClass: 'bg-destructive'},
}
</script>

<template>
  <span class="flex items-center gap-1.5">
    <span :class="`inline-block size-2 shrink-0 rounded-full ${STATUS_CONFIG[status].dotClass}`"/>
    <span class="text-xs text-muted-foreground">{{ STATUS_CONFIG[status].label }}</span>
  </span>
</template>