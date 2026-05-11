<script setup lang="ts">
import {computed} from 'vue'
import {useRoute} from 'vue-router'
import {useProjectStore} from '@/stores/project.store'
import {useBranchStore} from '@/stores/branch.store'
import {SidebarTrigger} from '@/components/ui/sidebar'
import {Button} from '@/components/ui/button'
import {useDark, useToggle} from '@vueuse/core'
import {Sun, Moon} from 'lucide-vue-next'
import {useCommandPalette} from '@/composables/useCommandPalette'

const route = useRoute()
const projectStore = useProjectStore()
const branchStore = useBranchStore()

const ROUTE_TITLES: Record<string, string> = {
  'projects.list': 'Projects',
  'settings.organization': 'Organization settings',
  'daemon': 'Daemon',
  'daemon.monitoring': 'Daemon — Monitoring',
  'projects.branches.metrics': 'Monitoring',
  'projects.branches.logs': 'Logs',
}

const title = computed(() => {
  const projectId = route.params.projectId as string | undefined
  const branchId = route.params.branchId as string | undefined

  if (route.name === 'daemon.logs') {
    const component = (route.params.component as string) ?? ''
    return `Daemon — Logs — ${component}`
  }

  if (projectId) {
    const project = projectStore.projects.find((p) => p.id === projectId)
    const projectName = project?.name ?? 'Project'
    if (branchId) {
      const branch = branchStore.branches.find((b) => b.id === branchId)
      const branchName = branch?.name ?? 'Branch'
      if (route.name === 'projects.branches.data') return `${projectName} / ${branchName} — Data`
      if (route.name === 'projects.branches.logs') {
        const component = (route.params.component as string) ?? ''
        return `${projectName} / ${branchName} — Logs — ${component}`
      }
      return `${projectName} / ${branchName}`
    }
    return route.name === 'projects.settings' ? `${projectName} — Settings` : projectName
  }
  return ROUTE_TITLES[route.name as string] ?? 'Page'
})

const isDark = useDark()
const toggleDark = useToggle(isDark)
const { open: commandOpen } = useCommandPalette()
</script>

<template>
  <header class="flex h-12 shrink-0 items-center gap-2 border-b px-2" :class="{dark: isDark}">
    <SidebarTrigger class="cursor-pointer"/>
    <span class="text-sm font-medium">{{ title }}</span>
    <Button variant="outline" size="sm" class="ml-auto cursor-pointer gap-2 text-muted-foreground" @click="commandOpen = true">
      <span class="text-sm">Search…</span>
      <kbd class="pointer-events-none hidden select-none items-center gap-1 rounded border bg-muted px-1.5 py-0.5 font-mono text-xs sm:flex">⌘K</kbd>
    </Button>
    <Button variant="ghost" size="icon" class="cursor-pointer" @click="toggleDark()">
      <Sun v-if="isDark" class="size-4"/>
      <Moon v-else class="size-4"/>
    </Button>
  </header>
</template>

<style>
.dark {
  color-scheme: dark;
}
</style>