<script setup lang="ts">
defineProps<{
  lastRecordLsn: string
  remoteConsistentLsn: string
}>()
</script>

<template>
  <span
    :class="`inline-flex cursor-default items-center rounded border px-1.5 py-0.5 text-xs font-medium ${
      remoteConsistentLsn === lastRecordLsn
        ? 'border-green-500/30 bg-green-500/10 text-green-600'
        : 'border-amber-500/30 bg-amber-500/10 text-amber-600'
    }`"
  >
    {{ remoteConsistentLsn === lastRecordLsn ? 'checkpointed' : 'awaiting checkpoint' }}
  </span>
</template>