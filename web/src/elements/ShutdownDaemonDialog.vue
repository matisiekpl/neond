<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDaemonStore } from '@/stores/daemon.store'
import { Loader2, TriangleAlert } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

const props = defineProps<{
  open: boolean
  awaitingCount: number
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const daemonStore = useDaemonStore()
const waitForCheckpoints = ref(true)

watch(() => props.open, (val) => {
  if (val) waitForCheckpoints.value = props.awaitingCount > 0
})

async function onConfirm() {
  await daemonStore.shutdown(waitForCheckpoints.value)
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Shutdown daemon?</DialogTitle>
        <DialogDescription>
          This stops all compute endpoints and the embedded pageserver/safekeeper.
        </DialogDescription>
      </DialogHeader>

      <div class="flex flex-col gap-3">
        <Alert v-if="awaitingCount > 0">
          <TriangleAlert class="size-4 text-amber-600" />
          <AlertDescription>
            {{ awaitingCount }} {{ awaitingCount === 1 ? 'branch is' : 'branches are' }} not checkpointed against the last received WAL record. Shutdown may not guarantee durability of data.
          </AlertDescription>
        </Alert>

        <label v-if="awaitingCount > 0" class="flex items-center gap-2 text-sm cursor-pointer">
          <input type="checkbox" v-model="waitForCheckpoints" class="cursor-pointer" />
          Wait for checkpoints before stopping
        </label>
      </div>

      <DialogFooter class="mt-4">
        <Button variant="outline" type="button" class="cursor-pointer" @click="emit('update:open', false)">
          Cancel
        </Button>
        <Button
          type="button"
          class="cursor-pointer bg-red-600 hover:bg-red-700 text-white"
          :disabled="daemonStore.shuttingDownSubmitting"
          @click="onConfirm"
        >
          <Loader2 v-if="daemonStore.shuttingDownSubmitting" class="mr-1.5 size-3.5 animate-spin" />
          Shutdown
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>