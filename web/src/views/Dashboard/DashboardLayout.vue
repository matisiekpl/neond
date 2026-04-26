<script setup lang="ts">
import { computed, watch, watchEffect } from 'vue'
import { useIntervalFn } from '@vueuse/core'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { useProjectStore } from '@/stores/project.store'
import { useBranchStore } from '@/stores/branch.store'
import { Loader2 } from 'lucide-vue-next'
import AppSidebar from '@/elements/AppSidebar.vue'
import AppMainHeader from '@/elements/AppMainHeader.vue'
import CommandPalette from '@/elements/CommandPalette.vue'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'

const authStore = useAuthStore()
const organizationStore = useOrganizationStore()
const projectStore = useProjectStore()
const branchStore = useBranchStore()
const route = useRoute()
const router = useRouter()

const isLoading = computed(() =>
  !authStore.initialized || (!!authStore.user && !organizationStore.loaded),
)

watch(
  () => route.params.organizationId as string,
  (organizationId) => {
    if (organizationId) organizationStore.saveSelected(organizationId)
  },
  { immediate: true },
)

watchEffect(() => {
  if (isLoading.value) return
  if (!authStore.user) {
    router.replace({ name: 'login' })
    return
  }
  if (organizationStore.organizations.length === 0) {
    router.replace({ name: 'setup-organization' })
    return
  }
  const organizationId = route.params.organizationId as string
  const valid = organizationStore.organizations.some((o) => o.id === organizationId)
  if (!valid) {
    router.replace({ name: 'projects.list', params: { organizationId: organizationStore.organizations[0].id } })
  }
})

const { pause: pauseBranchPoll, resume: resumeBranchPoll } = useIntervalFn(
  () => {
    const orgId = organizationStore.selectedOrganizationId
    const projectId = route.params.projectId as string | undefined
    if (orgId && projectId) branchStore.fetch(orgId, projectId, true)
  },
  500,
  { immediate: false },
)

watch(
  () => organizationStore.selectedOrganizationId,
  (organizationId) => {
    if (organizationId) projectStore.fetch(organizationId, true)
  },
  { immediate: true },
)

watch(
  [() => organizationStore.selectedOrganizationId, () => route.params.projectId as string | undefined],
  ([orgId, projectId]) => {
    pauseBranchPoll()
    if (!orgId || !projectId) return
    branchStore.fetch(orgId, projectId)
    resumeBranchPoll()
  },
  { immediate: true },
)

const showLayout = computed(
  () =>
    !isLoading.value &&
    !!authStore.user &&
    organizationStore.organizations.length > 0 &&
    organizationStore.organizations.some((o) => o.id === route.params.organizationId),
)
</script>

<template>
  <div v-if="isLoading" class="flex min-h-screen items-center justify-center">
    <Loader2 class="size-8 animate-spin" />
  </div>

  <template v-else-if="showLayout">
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <AppMainHeader />
        <div class="flex min-h-0 flex-1 flex-col overflow-auto p-4 md:p-6">
          <RouterView />
        </div>
      </SidebarInset>
    </SidebarProvider>
    <CommandPalette />
  </template>
</template>