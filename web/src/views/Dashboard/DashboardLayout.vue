<script setup lang="ts">
import { computed, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { Loader2 } from 'lucide-vue-next'
import AppSidebar from '@/elements/AppSidebar.vue'
import AppMainHeader from '@/elements/AppMainHeader.vue'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'

const authStore = useAuthStore()
const organizationStore = useOrganizationStore()
const route = useRoute()
const router = useRouter()

const onSetupRoute = computed(() => route.path === '/dashboard/setup-organization')

const isLoading = computed(() =>
  !authStore.initialized || (!!authStore.user && !organizationStore.loaded),
)

watchEffect(() => {
  if (isLoading.value) return
  if (!authStore.user) {
    router.replace('/login')
    return
  }
  if (!onSetupRoute.value && organizationStore.organizations.length === 0) {
    router.replace('/dashboard/setup-organization')
    return
  }
  if (onSetupRoute.value && organizationStore.organizations.length > 0) {
    router.replace('/dashboard')
  }
})

const showLayout = computed(() =>
  !isLoading.value &&
  !!authStore.user &&
  !(!onSetupRoute.value && organizationStore.organizations.length === 0) &&
  !(onSetupRoute.value && organizationStore.organizations.length > 0),
)

const showSetupOutlet = computed(() =>
  showLayout.value && onSetupRoute.value,
)

const showFullLayout = computed(() =>
  showLayout.value && !onSetupRoute.value,
)
</script>

<template>
  <div v-if="isLoading" class="flex min-h-screen items-center justify-center">
    <Loader2 class="size-8 animate-spin" />
  </div>

  <template v-else-if="showSetupOutlet">
    <RouterView />
  </template>

  <template v-else-if="showFullLayout">
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <AppMainHeader />
        <div class="flex min-h-0 flex-1 flex-col overflow-auto p-4 md:p-6">
          <RouterView />
        </div>
      </SidebarInset>
    </SidebarProvider>
  </template>
</template>
