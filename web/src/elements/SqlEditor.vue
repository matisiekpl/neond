<script setup lang="ts">
import { Play, Square } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'

const props = defineProps<{
  modelValue: string
  loading: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  run: []
  cancel: []
}>()

function onKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault()
    emit('run')
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="shrink-0 border-b px-3 py-2 flex items-center justify-between gap-2">
      <span class="text-xs text-muted-foreground font-mono">SQL Editor</span>
      <div class="flex items-center gap-2">
        <span class="text-xs text-muted-foreground hidden sm:inline">⌘↵</span>
        <Button
          v-if="props.loading"
          size="sm"
          variant="destructive"
          class="cursor-pointer"
          @click="emit('cancel')"
        >
          <Square class="size-3" />
          Cancel
        </Button>
        <Button
          v-else
          size="sm"
          class="cursor-pointer bg-green-600 hover:bg-green-700 text-white"
          :disabled="!props.modelValue.trim()"
          @click="emit('run')"
        >
          <Play class="size-3" />
          Run
        </Button>
      </div>
    </div>
    <textarea
      :value="props.modelValue"
      class="flex-1 min-h-0 resize-none bg-transparent p-3 leading-6 outline-none font-mono text-sm"
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      placeholder="SELECT ..."
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
      @keydown="onKeydown"
    />
  </div>
</template>