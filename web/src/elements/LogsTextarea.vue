<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue'
import { useLogsStore } from '@/stores/logs.store'
import { Button } from '@/components/ui/button'
import { Copy, Trash2 } from 'lucide-vue-next'

const logsStore = useLogsStore()
const containerRef = ref<HTMLPreElement | null>(null)
const isAtBottom = ref(true)

const text = computed(() =>
  logsStore.lines
    .map((line) => `${line.timestamp} [${line.stream}] ${line.message}`)
    .join('\n'),
)

function onScroll() {
  const el = containerRef.value
  if (!el) return
  isAtBottom.value = el.scrollHeight - el.scrollTop - el.clientHeight < 32
}

watch(
  () => logsStore.lines.length,
  async () => {
    if (!isAtBottom.value) return
    await nextTick()
    const el = containerRef.value
    if (el) el.scrollTop = el.scrollHeight
  },
)

function copyLogs() {
  navigator.clipboard.writeText(text.value)
}

function clearLogs() {
  logsStore.lines.length = 0
  logsStore.lines.splice(0)
}
</script>

<template>
  <div class="flex flex-col h-full gap-2">
    <div class="flex items-center justify-between">
      <span class="text-sm text-muted-foreground">
        {{ logsStore.connected ? 'Connected' : 'Disconnected' }} &mdash;
        {{ logsStore.lines.length }} line(s)
      </span>
      <div class="flex gap-2">
        <Button variant="outline" size="sm" class="cursor-pointer" @click="copyLogs">
          <Copy class="size-4" />
          Copy
        </Button>
        <Button variant="outline" size="sm" class="cursor-pointer" @click="clearLogs">
          <Trash2 class="size-4" />
          Clear
        </Button>
      </div>
    </div>
    <pre
      ref="containerRef"
      class="flex-1 overflow-auto rounded-md border bg-black text-green-400 font-mono text-xs p-4 whitespace-pre leading-5"
      @scroll="onScroll"
    >{{ text || 'No logs yet.' }}</pre>
  </div>
</template>