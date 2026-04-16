<script setup lang="ts">
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { useTitle } from '@vueuse/core'
import { useRouter } from 'vue-router'
import { useOrganizationStore } from '@/stores/organization.store'
import { getAppError } from '@/api/utils'
import { Loader2 } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

useTitle('Create organization — neond')
const router = useRouter()
const organizationStore = useOrganizationStore()

const name = ref('')
const submitting = ref(false)

async function onSubmit() {
  const trimmed = name.value.trim()
  if (!trimmed) return
  submitting.value = true
  try {
    const organization = await organizationStore.create(trimmed)
    router.push({ name: 'projects.list', params: { organizationId: organization.id } })
  } catch (err) {
    toast.error(getAppError(err))
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <main class="flex min-h-svh w-full flex-col items-center justify-center bg-background px-4">
    <form @submit.prevent="onSubmit" class="w-full max-w-sm space-y-6">
      <div class="space-y-2 text-center">
        <h1 class="text-xl font-semibold tracking-tight">Name your organization</h1>
        <p class="text-sm text-muted-foreground">
          You need an organization before you can continue.
        </p>
      </div>
      <div class="space-y-2">
        <Label for="setup-org-name">Organization name</Label>
        <Input
          id="setup-org-name"
          v-model="name"
          placeholder="Acme Inc."
          autocomplete="organization"
          autofocus
        />
      </div>
      <Button type="submit" class="w-full cursor-pointer" :disabled="submitting || !name.trim()">
        <Loader2 v-if="submitting" class="mr-1.5 size-3.5 animate-spin" />
        Continue
      </Button>
    </form>
  </main>
</template>
