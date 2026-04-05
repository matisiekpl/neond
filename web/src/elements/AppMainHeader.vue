<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useProjectStore } from '@/stores/project.store'
import { SidebarTrigger } from '@/components/ui/sidebar'

const route = useRoute()
const projectStore = useProjectStore()

const TITLES: Record<string, string> = {
  '/dashboard': 'Dashboard',
  '/dashboard/projects': 'Projects',
  '/dashboard/settings/organization': 'Organization settings',
}

const title = computed(() => {
  const projectId = route.params.projectId as string | undefined
  if (projectId) {
    const project = projectStore.projects.find((p) => p.id === projectId)
    const projectName = project?.name ?? 'Project'
    return route.path.endsWith('/settings') ? `${projectName} — Settings` : projectName
  }
  return TITLES[route.path] ?? 'Page'
})
</script>

<template>
  <header class="flex h-12 shrink-0 items-center gap-2 border-b px-2">
    <SidebarTrigger class="cursor-pointer"/>
    <span class="text-sm font-medium">{{ title }}</span>
  </header>
</template>
