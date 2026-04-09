<script setup lang="ts">
import { ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import { useRouter } from 'vue-router'
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

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const router = useRouter()
const organizationStore = useOrganizationStore()

const name = ref('')
const submitting = ref(false)

watch(() => props.open, (val) => {
  if (!val) name.value = ''
})

async function onSubmit() {
  const trimmed = name.value.trim()
  if (!trimmed) return
  submitting.value = true
  try {
    const organization = await organizationStore.create(trimmed)
    emit('update:open', false)
    router.push({ name: 'projects.list', params: { organizationId: organization.id } })
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
        <DialogTitle>Create organization</DialogTitle>
        <DialogDescription>
          Add a new organization. You will be added as a member automatically.
        </DialogDescription>
      </DialogHeader>
      <form @submit.prevent="onSubmit">
        <div class="grid gap-2">
          <Label for="create-org-name">Name</Label>
          <Input
            id="create-org-name"
            v-model="name"
            autofocus
            placeholder="Organization name"
          />
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
