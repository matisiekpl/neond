<script setup lang="ts">
import { ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import { useProjectStore } from '@/stores/project.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { getAppError } from '@/api/utils'
import { Loader2 } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const PG_VERSIONS = ['V17', 'V16', 'V15', 'V14'] as const

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const projectStore = useProjectStore()
const organizationStore = useOrganizationStore()

const name = ref('')
const pgVersion = ref('V17')
const submitting = ref(false)

watch(() => props.open, (val) => {
  if (!val) {
    name.value = ''
    pgVersion.value = 'V17'
  }
})

async function onSubmit() {
  const trimmed = name.value.trim()
  if (!trimmed || !organizationStore.selectedOrganizationId) return
  submitting.value = true
  try {
    await projectStore.createProject(organizationStore.selectedOrganizationId, { name: trimmed, pg_version: pgVersion.value })
    emit('update:open', false)
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Create project</DialogTitle>
        <DialogDescription>
          A project contains branches and compute endpoints.
        </DialogDescription>
      </DialogHeader>
      <form @submit.prevent="onSubmit">
        <div class="grid gap-4">
          <div class="grid gap-2">
            <Label for="create-project-name">Name</Label>
            <Input
              id="create-project-name"
              v-model="name"
              autofocus
              placeholder="my-project"
            />
          </div>
          <div class="grid gap-2">
            <Label for="create-project-pg">PostgreSQL version</Label>
            <select
              id="create-project-pg"
              v-model="pgVersion"
              class="flex h-9 w-full border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              <option v-for="v in PG_VERSIONS" :key="v" :value="v">
                PostgreSQL {{ v.slice(1) }}
              </option>
            </select>
          </div>
        </div>
        <DialogFooter class="mt-4">
          <Button variant="outline" type="button" class="cursor-pointer" @click="emit('update:open', false)">
            Cancel
          </Button>
          <Button type="submit" class="cursor-pointer" :disabled="submitting || !name.trim()">
            <Loader2 v-if="submitting" class="mr-1.5 size-3.5 animate-spin" />
            Create
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
