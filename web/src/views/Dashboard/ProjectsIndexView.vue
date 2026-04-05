<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useTitle } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { MoreHorizontal, Loader2 } from 'lucide-vue-next'
import { useProjectStore } from '@/stores/project.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { getAppError } from '@/api/utils'
import CreateProjectDialog from '@/elements/CreateProjectDialog.vue'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

useTitle('Projects — neond')
const projectStore = useProjectStore()
const organizationStore = useOrganizationStore()

const createOpen = ref(false)
const renameOpen = ref(false)
const renameId = ref<string | null>(null)
const renameName = ref('')
const renameSubmitting = ref(false)
const deleteOpen = ref(false)
const deleteId = ref<string | null>(null)
const deleteSubmitting = ref(false)

watch(() => organizationStore.selectedOrganizationId, (id) => {
  if (id) projectStore.fetchProjects(id)
}, { immediate: true })

watch(renameOpen, (val) => {
  if (!val) renameName.value = ''
})

function openRename(id: string, currentName: string) {
  renameId.value = id
  renameName.value = currentName
  renameOpen.value = true
}

function openDelete(id: string) {
  deleteId.value = id
  deleteOpen.value = true
}

async function submitRename() {
  if (!organizationStore.selectedOrganizationId || !renameId.value) return
  const trimmed = renameName.value.trim()
  if (!trimmed) return
  renameSubmitting.value = true
  try {
    await projectStore.updateProject(organizationStore.selectedOrganizationId, renameId.value, { name: trimmed })
    renameOpen.value = false
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    renameSubmitting.value = false
  }
}

async function confirmDelete() {
  if (!organizationStore.selectedOrganizationId || !deleteId.value) return
  deleteSubmitting.value = true
  try {
    await projectStore.deleteProject(organizationStore.selectedOrganizationId, deleteId.value)
    deleteOpen.value = false
  } finally {
    deleteSubmitting.value = false
  }
}

function formatDate(d: string) {
  return new Date(d).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-lg font-semibold">Projects</h1>
        <p class="text-sm text-muted-foreground">Manage your database projects.</p>
      </div>
      <Button type="button" class="cursor-pointer" @click="createOpen = true">New project</Button>
    </div>

    <div v-if="projectStore.loading" class="flex justify-center py-12">
      <Loader2 class="size-6 animate-spin" />
    </div>

    <section
      v-else-if="projectStore.projects.length === 0"
      class="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center"
    >
      <p class="text-sm font-medium">No projects yet</p>
      <p class="mt-1 text-sm text-muted-foreground">Create your first project to get started.</p>
      <Button type="button" class="mt-4 cursor-pointer" @click="createOpen = true">New project</Button>
    </section>

    <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <div v-for="project in projectStore.projects" :key="project.id" class="relative">
        <RouterLink :to="`/dashboard/projects/${project.id}`" class="block">
          <Card class="transition-colors hover:bg-accent/50">
            <CardHeader class="pb-2 pr-10">
              <CardTitle class="truncate text-base">{{ project.name }}</CardTitle>
              <CardDescription>PostgreSQL {{ project.pg_version.replace(/^V/i, '') }}</CardDescription>
            </CardHeader>
            <CardContent>
              <p class="text-xs text-muted-foreground">Created {{ formatDate(project.created_at) }}</p>
            </CardContent>
          </Card>
        </RouterLink>
        <div class="absolute right-3 top-3">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button
                variant="ghost"
                size="icon"
                class="size-7 cursor-pointer"
                @click.prevent
              >
                <MoreHorizontal class="size-4" />
                <span class="sr-only">Open menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem @click="openRename(project.id, project.name)">Rename</DropdownMenuItem>
              <DropdownMenuItem
                class="text-destructive focus:text-destructive"
                @click="openDelete(project.id)"
              >
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>

    <CreateProjectDialog v-model:open="createOpen" />

    <Dialog v-model:open="renameOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Rename project</DialogTitle>
        </DialogHeader>
        <form @submit.prevent="submitRename">
          <div class="grid gap-2 py-2">
            <Label for="rename-project-name">Name</Label>
            <Input id="rename-project-name" v-model="renameName" />
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="renameOpen = false">Cancel</Button>
            <Button type="submit" class="cursor-pointer" :disabled="renameSubmitting || !renameName.trim()">Save</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <AlertDialog v-model:open="deleteOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete project?</AlertDialogTitle>
          <AlertDialogDescription>
            All branches and data in this project will be permanently removed. This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="deleteSubmitting">Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90 cursor-pointer"
            :disabled="deleteSubmitting"
            @click="confirmDelete"
          >
            <Loader2 v-if="deleteSubmitting" class="mr-1.5 size-3.5 animate-spin" />
            Delete project
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
