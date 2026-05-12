<script setup lang="ts">
import {computed, watch} from 'vue'
import {Copy, TriangleAlert} from 'lucide-vue-next'
import {toast} from 'vue-sonner'
import {Button} from '@/components/ui/button'
import {Label} from '@/components/ui/label'
import {Switch} from '@/components/ui/switch'
import {Textarea} from '@/components/ui/textarea'
import {Alert, AlertDescription} from '@/components/ui/alert'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import type {Branch} from '@/types/models/branch'

const props = defineProps<{
  open: boolean
  branch: Branch | null
  pooled: boolean
  libcompat: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'update:pooled': [value: boolean]
  'update:libcompat': [value: boolean]
}>()

const poolerAvailable = computed(() => !!props.branch?.pooler_connection_string)

watch(
  () => [props.open, poolerAvailable.value] as const,
  ([open, available]) => {
    if (open && !available && props.pooled) {
      emit('update:pooled', false)
    }
  },
)

const connectionString = computed(() => {
  if (!props.branch) return ''
  const base = props.pooled
    ? props.branch.pooler_connection_string
    : props.branch.connection_string
  if (!base) return ''
  if (!props.libcompat) return base
  const separator = base.includes('?') ? '&' : '?'
  return `${base}${separator}uselibpqcompat=true`
})

async function copy() {
  if (!connectionString.value) return
  await navigator.clipboard.writeText(connectionString.value)
  toast.success('Connection string copied')
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-2xl">
      <DialogHeader>
        <DialogTitle>Connect to your database</DialogTitle>
      </DialogHeader>

      <div class="flex flex-col gap-4">
        <div class="flex flex-wrap items-center gap-x-6 gap-y-3">
          <div class="flex items-center gap-2">
            <Switch
              id="connect-pooled"
              :model-value="pooled"
              :disabled="!poolerAvailable"
              @update:model-value="emit('update:pooled', $event)"
            />
            <Label
              for="connect-pooled"
              :class="poolerAvailable ? 'cursor-pointer' : 'cursor-not-allowed opacity-50'"
            >
              Connection pooling
              <span v-if="!poolerAvailable" class="text-xs text-muted-foreground">(unavailable)</span>
            </Label>
          </div>
          <div class="flex items-center gap-2">
            <Switch
              id="connect-libcompat"
              :model-value="libcompat"
              @update:model-value="emit('update:libcompat', $event)"
            />
            <Label for="connect-libcompat" class="cursor-pointer">libpq compatibility</Label>
          </div>
        </div>

        <Alert v-if="pooled">
          <TriangleAlert class="size-4 text-amber-600"/>
          <AlertDescription>
            Pooled connections do not enforce channel binding, which can leave the client open to
            MITM attacks. Use the direct connection if you need channel binding.
          </AlertDescription>
        </Alert>

        <Textarea
          :model-value="connectionString"
          readonly
          class="font-mono text-xs min-h-24"
        />
      </div>

      <DialogFooter class="mt-2">
        <Button
          variant="outline"
          type="button"
          class="cursor-pointer"
          @click="emit('update:open', false)"
        >
          Close
        </Button>
        <Button
          type="button"
          class="cursor-pointer"
          :disabled="!connectionString"
          @click="copy"
        >
          <Copy class="size-3.5"/>
          Copy snippet
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>