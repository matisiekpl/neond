<script setup lang="ts">
import AppSidebar from '@/components/AppSidebar.vue'
import {SidebarInset, SidebarProvider, SidebarTrigger} from '@/components/ui/sidebar'
import {Separator} from '@/components/ui/separator'
import {onMounted, watch} from 'vue'
import {useAuthStore} from '@/stores/auth.store.ts'
import {useOrganizationStore} from '@/stores/organization.store.ts'
import {useRoute} from 'vue-router'

const authStore = useAuthStore()
const organizationStore = useOrganizationStore()
const route = useRoute()

onMounted(async () => {
  if (!authStore.user) {
    await authStore.init()
  }
  if (authStore.user) {
    await organizationStore.ensureOrganizations(authStore.user.name)
  }
})

watch(
  () => authStore.user?.name,
  async (authenticatedUserDisplayName) => {
    if (authenticatedUserDisplayName) {
      await organizationStore.ensureOrganizations(authenticatedUserDisplayName)
    }
  },
)
</script>

<template>
  <SidebarProvider v-if="authStore.user">
    <AppSidebar />
    <SidebarInset class="min-h-0 overflow-hidden">
      <header class="flex h-14 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger class="-ml-1" />
        <Separator orientation="vertical" class="mr-2 h-4" />
        <span class="text-sm text-muted-foreground truncate">{{ route.meta.title ?? '' }}</span>
      </header>
      <div class="flex min-h-0 flex-1 flex-col overflow-auto p-4 md:p-6">
        <RouterView />
      </div>
    </SidebarInset>
  </SidebarProvider>
</template>
