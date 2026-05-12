<script setup lang="ts">
import {ref, watch} from 'vue'
import {Loader2} from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {Button} from '@/components/ui/button'
import {Input} from '@/components/ui/input'
import {Label} from '@/components/ui/label'
import {Textarea} from '@/components/ui/textarea'
import {useBranchStore} from '@/stores/branch.store'

const props = defineProps<{
  open: boolean
  organizationId: string
  projectId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const branchStore = useBranchStore()

const name = ref('')
const sourceConnectionString = ref('')
const submitting = ref(false)

watch(
  () => props.open,
  (value) => {
    if (value) {
      name.value = ''
      sourceConnectionString.value = ''
    }
  },
)

async function submit() {
  const trimmedName = name.value.trim()
  const trimmedConnectionString = sourceConnectionString.value.trim()
  if (!trimmedName || !trimmedConnectionString) return
  submitting.value = true
  try {
    await branchStore.importBranch(props.organizationId, props.projectId, trimmedName, trimmedConnectionString)
    emit('update:open', false)
  } catch {
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>Import from external Postgres</DialogTitle>
      </DialogHeader>
      <form @submit.prevent="submit">
        <div class="grid gap-4 py-2">
          <div class="grid gap-2">
            <Label for="import-branch-name">Name</Label>
            <Input id="import-branch-name" v-model="name" placeholder="my-branch" autofocus/>
          </div>
          <div class="grid gap-2">
            <Label for="import-branch-source">Source connection string</Label>
            <Textarea
              id="import-branch-source"
              v-model="sourceConnectionString"
              class="font-mono text-xs min-h-24"
              placeholder="postgres://user:password@host:5432/dbname"
            />
          </div>
        </div>
        <DialogFooter class="mt-2">
          <Button variant="outline" type="button" class="cursor-pointer" @click="emit('update:open', false)">Cancel</Button>
          <Button
            type="submit"
            class="cursor-pointer"
            :disabled="submitting || !name.trim() || !sourceConnectionString.trim()"
          >
            <Loader2 v-if="submitting" class="mr-1.5 size-3.5 animate-spin"/>
            Start import
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
