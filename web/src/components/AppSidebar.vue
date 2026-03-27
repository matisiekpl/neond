<script setup lang="ts">
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from '@/components/ui/sidebar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {Button} from '@/components/ui/button'
import {Input} from '@/components/ui/input'
import {Label} from '@/components/ui/label'
import {ChevronDown, GalleryVerticalEnd, LogOut, Plus, Settings} from 'lucide-vue-next'
import {RouterLink} from 'vue-router'
import {computed, ref, watch} from 'vue'
import {useOrganizationStore} from '@/stores/organization.store.ts'
import {useAuthStore} from '@/stores/auth.store.ts'
import {getAppError} from '@/api/utils.ts'
import {toast} from 'vue-sonner'

const organizationStore = useOrganizationStore()
const authStore = useAuthStore()

const organizationCreateDialogOpen = ref(false)
const newOrganizationName = ref('')
const isOrganizationCreating = ref(false)

const currentOrganizationLabel = computed(
  () => organizationStore.currentOrganization?.name ?? 'Select organization',
)

watch(organizationCreateDialogOpen, (open) => {
  if (!open) newOrganizationName.value = ''
})

async function createOrganization() {
  const name = newOrganizationName.value.trim()
  if (!name) return
  isOrganizationCreating.value = true
  try {
    await organizationStore.createOrganization(name)
    organizationCreateDialogOpen.value = false
  } catch (e) {
    toast.error(getAppError(e))
  } finally {
    isOrganizationCreating.value = false
  }
}

function selectOrganization(organizationId: string) {
  organizationStore.saveSelectedOrganization(organizationId)
}
</script>

<template>
  <Sidebar collapsible="icon" variant="inset">
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <SidebarMenuButton size="lg" class="data-[state=open]:bg-sidebar-accent">
                <div
                  class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
                >
                  <GalleryVerticalEnd class="size-4" />
                </div>
                <div class="grid flex-1 text-left text-sm leading-tight">
                  <span class="truncate font-semibold">{{ currentOrganizationLabel }}</span>
                  <span class="truncate text-xs text-muted-foreground">Organization</span>
                </div>
                <ChevronDown class="ml-auto size-4" />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent class="min-w-56 rounded-lg" align="start" side="bottom">
              <DropdownMenuLabel class="text-xs text-muted-foreground">Organizations</DropdownMenuLabel>
              <DropdownMenuItem
                v-for="organization in organizationStore.organizations"
                :key="organization.id"
                class="gap-2 p-2"
                @click="selectOrganization(organization.id)"
              >
                <div class="flex size-6 items-center justify-center rounded-sm border">
                  <GalleryVerticalEnd class="size-4 shrink-0" />
                </div>
                <span class="truncate">{{ organization.name }}</span>
                <span
                  v-if="organization.id === organizationStore.selectedOrganizationId"
                  class="ml-auto text-xs text-muted-foreground"
                >
                  ✓
                </span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem class="gap-2 p-2" @click="organizationCreateDialogOpen = true">
                <div class="flex size-6 items-center justify-center rounded-md border bg-transparent">
                  <Plus class="size-4" />
                </div>
                <span class="font-medium text-muted-foreground">Create organization</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>Platform</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton as-child tooltip="Dashboard">
                <RouterLink to="/dashboard">
                  <GalleryVerticalEnd />
                  <span>Dashboard</span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton as-child tooltip="Organization settings">
                <RouterLink to="/dashboard/settings/organization">
                  <Settings />
                  <span>Organization</span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarFooter>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton class="text-muted-foreground" @click="authStore.logout()">
            <LogOut />
            <span>Log out</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarFooter>
    <SidebarRail />
  </Sidebar>

  <Dialog v-model:open="organizationCreateDialogOpen">
    <DialogContent class="sm:max-w-md" @open-auto-focus.prevent>
      <DialogHeader>
        <DialogTitle>Create organization</DialogTitle>
        <DialogDescription>Add a new organization. You will be added as a member automatically.</DialogDescription>
      </DialogHeader>
      <div class="grid gap-4 py-2">
        <div class="grid gap-2">
          <Label for="new-organization-name">Name</Label>
          <Input
            id="new-organization-name"
            v-model="newOrganizationName"
            placeholder="Acme Inc"
            @keydown.enter.prevent="createOrganization"
          />
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" type="button" @click="organizationCreateDialogOpen = false">Cancel</Button>
        <Button
          type="button"
          :disabled="isOrganizationCreating || !newOrganizationName.trim()"
          @click="createOrganization"
        >
          Create
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
