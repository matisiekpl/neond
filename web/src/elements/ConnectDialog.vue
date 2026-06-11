<script setup lang="ts">
import {computed} from 'vue'
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
  libcompat: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'update:libcompat': [value: boolean]
}>()

function withLibcompat(base: string | null | undefined): string {
  if (!base) return ''
  if (!props.libcompat) return base
  const separator = base.includes('?') ? '&' : '?'
  return `${base}${separator}uselibpqcompat=true`
}

const directConnectionString = computed(() => withLibcompat(props.branch?.connection_string))
const pooledConnectionString = computed(() => withLibcompat(props.branch?.pooler_connection_string))

async function copy(value: string) {
  if (!value) return
  await navigator.clipboard.writeText(value)
  toast.success('Connection string copied')
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-2xl">
      <DialogHeader>
        <DialogTitle>Connect to your database</DialogTitle>
      </DialogHeader>

      <div class="flex flex-col gap-6">
        <div class="flex items-center gap-2">
          <Switch
            id="connect-libcompat"
            :model-value="libcompat"
            @update:model-value="emit('update:libcompat', $event)"
          />
          <Label for="connect-libcompat" class="cursor-pointer">libpq compatibility</Label>
        </div>

        <div class="flex flex-col gap-2">
          <Label>Direct connection</Label>
          <Textarea
            :model-value="directConnectionString"
            readonly
            class="font-mono text-xs min-h-24"
          />
          <div class="flex justify-end">
            <Button
              variant="outline"
              type="button"
              size="sm"
              class="cursor-pointer"
              :disabled="!directConnectionString"
              @click="copy(directConnectionString)"
            >
              <Copy class="size-3.5"/>
              Copy
            </Button>
          </div>
        </div>

        <div v-if="pooledConnectionString" class="flex flex-col gap-2">
          <Label class="mb-1">Connection pooling (pgbouncer)</Label>
          <Alert>
            <TriangleAlert class="size-4 text-amber-600"/>
            <AlertDescription>
              Pooled connections do not enforce channel binding, which can leave the client open to
              MITM attacks. Use the direct connection if you need channel binding.
            </AlertDescription>
          </Alert>
          <Textarea
            :model-value="pooledConnectionString"
            readonly
            class="font-mono text-xs min-h-24"
          />
          <div class="flex justify-end">
            <Button
              variant="outline"
              type="button"
              size="sm"
              class="cursor-pointer"
              @click="copy(pooledConnectionString)"
            >
              <Copy class="size-3.5"/>
              Copy
            </Button>
          </div>
        </div>
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
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
