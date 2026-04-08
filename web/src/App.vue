<script setup lang="ts">
import { watch } from 'vue'
import { RouterView } from 'vue-router'
import { Toaster } from '@/components/ui/sonner'
import { onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth.store'
import { useOrganizationStore } from '@/stores/organization.store'

const authStore = useAuthStore()
const organizationStore = useOrganizationStore()

onMounted(async () => {
  await authStore.bootstrap()
})

watch(() => authStore.user?.id, async (userId) => {
  if (!userId) {
    organizationStore.reset()
    return
  }
  if (!organizationStore.loaded) {
    await organizationStore.loadOrganizations()
  }
})
</script>

<template>
  <RouterView />
  <Toaster />
</template>
