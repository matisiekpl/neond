<script setup lang="ts">
import {computed} from 'vue'
import {useRoute} from 'vue-router'
import {useProjectStore} from '@/stores/project.store'
import {SidebarTrigger} from '@/components/ui/sidebar'
import {Button} from '@/components/ui/button'
import {useDark, useToggle} from '@vueuse/core'
import {Sun, Moon} from 'lucide-vue-next'

const route = useRoute()
const projectStore = useProjectStore()

const ROUTE_TITLES: Record<string, string> = {
  'projects.list': 'Projects',
  'settings.organization': 'Organization settings',
  'daemon': 'Daemon',
}

const title = computed(() => {
  const projectId = route.params.projectId as string | undefined
  if (projectId) {
    const project = projectStore.projects.find((p) => p.id === projectId)
    const projectName = project?.name ?? 'Project'
    return route.name === 'projects.settings' ? `${projectName} — Settings` : projectName
  }
  return ROUTE_TITLES[route.name as string] ?? 'Page'
})

const isDark = useDark()
const toggleDark = useToggle(isDark)
</script>

<template>
  <header class="flex h-12 shrink-0 items-center gap-2 border-b px-2">
    <SidebarTrigger class="cursor-pointer"/>
    <span class="text-sm font-medium">{{ title }}</span>
    <Button variant="ghost" size="icon" class="ml-auto cursor-pointer" @click="toggleDark()">
      <Sun v-if="isDark" class="size-4"/>
      <Moon v-else class="size-4"/>
    </Button>
  </header>
</template>