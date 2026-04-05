<script setup lang="ts">
import {Input} from '@/components/ui/input'
import {useVModel} from "@vueuse/core";

const props = defineProps<{
  modelValue: string
}>()
const emit = defineEmits(['update:modelValue'])
const data = useVModel(props, 'modelValue', emit)

const MB = 1024 * 1024

function bytesToMb(bytes: string): string {
  if (!bytes.trim()) return ''
  const n = Number(bytes)
  if (isNaN(n)) return ''
  return String(n / MB)
}

function mbToBytes(mb: string): string {
  if (!mb.trim()) return ''
  const n = Number(mb)
  if (isNaN(n)) return ''
  return String(Math.round(n * MB))
}

function onUpdate(v: string | number) {
  data.value = mbToBytes(String(v))
}
</script>

<template>
  <div class="flex items-center gap-2">
    <Input
      type="number"
      :min="0"
      :model-value="bytesToMb(data)"
      @update:model-value="onUpdate"
    />
    <span class="cursor-pointer shrink-0 text-sm text-muted-foreground">MB</span>
  </div>
</template>