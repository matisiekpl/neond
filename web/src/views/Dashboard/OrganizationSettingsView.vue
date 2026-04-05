<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useTitle } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { Loader2 } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { getAppError } from '@/api/utils'
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
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

useTitle('Organization settings — neond')
const authStore = useAuthStore()
const organizationStore = useOrganizationStore()

const currentOrganization = computed(() =>
  organizationStore.selectedOrganizationId
    ? organizationStore.organizations.find((o) => o.id === organizationStore.selectedOrganizationId)
    : undefined,
)

const orgName = ref('')
const orgNameOriginal = ref('')
const nameSubmitting = ref(false)

const inviteOpen = ref(false)
const inviteEmail = ref('')
const inviteSubmitting = ref(false)

const removeOpen = ref(false)
const removeUserId = ref<string | null>(null)
const removeSubmitting = ref(false)

const deleteOpen = ref(false)
const deleteSubmitting = ref(false)

watch(currentOrganization, (org) => {
  if (org) {
    orgName.value = org.name
    orgNameOriginal.value = org.name
  }
}, { immediate: true })

watch(() => organizationStore.selectedOrganizationId, (id) => {
  if (id) organizationStore.fetchMembers(id)
}, { immediate: true })

watch(inviteOpen, (val) => {
  if (!val) inviteEmail.value = ''
})

const nameIsDirty = computed(() => orgName.value.trim() !== orgNameOriginal.value)

async function saveName() {
  if (!organizationStore.selectedOrganizationId) return
  const trimmed = orgName.value.trim()
  if (!trimmed) return
  nameSubmitting.value = true
  try {
    await organizationStore.updateOrganization(organizationStore.selectedOrganizationId, trimmed)
    orgNameOriginal.value = trimmed
  } catch (err) {
    toast.error(getAppError(err))
  } finally {
    nameSubmitting.value = false
  }
}

async function submitInvite() {
  if (!organizationStore.selectedOrganizationId) return
  const trimmed = inviteEmail.value.trim()
  if (!trimmed) return
  inviteSubmitting.value = true
  try {
    await organizationStore.addMemberByEmail(organizationStore.selectedOrganizationId, trimmed)
    inviteOpen.value = false
  } catch {}
  finally {
    inviteSubmitting.value = false
  }
}

function openRemove(userId: string) {
  removeUserId.value = userId
  removeOpen.value = true
}

async function confirmRemove() {
  if (!organizationStore.selectedOrganizationId || !removeUserId.value) return
  removeSubmitting.value = true
  try {
    await organizationStore.removeMember(organizationStore.selectedOrganizationId, removeUserId.value)
    removeOpen.value = false
    removeUserId.value = null
  } catch {}
  finally {
    removeSubmitting.value = false
  }
}

async function confirmDelete() {
  if (!organizationStore.selectedOrganizationId) return
  deleteSubmitting.value = true
  try {
    await organizationStore.deleteOrganization(organizationStore.selectedOrganizationId)
    deleteOpen.value = false
    await organizationStore.loadOrganizations()
  } catch (err) {
    toast.error(getAppError(err))
  } finally {
    deleteSubmitting.value = false
  }
}
</script>

<template>
  <section
    v-if="!currentOrganization"
    class="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center"
  >
    <p class="text-sm text-muted-foreground">No organization selected.</p>
  </section>

  <div v-else class="w-full space-y-6">
    <Card>
      <CardHeader>
        <CardTitle>General</CardTitle>
        <CardDescription>Update your organization display name.</CardDescription>
      </CardHeader>
      <CardContent>
        <form @submit.prevent="saveName">
          <div class="space-y-2">
            <Label for="organization-name">Name</Label>
            <Input id="organization-name" v-model="orgName" class="w-full" />
          </div>
          <Button
            type="submit"
            class="mt-4 cursor-pointer"
            :disabled="nameSubmitting || !orgName.trim() || !nameIsDirty"
          >
            <Loader2 v-if="nameSubmitting" class="mr-1.5 size-3.5 animate-spin" />
            Save changes
          </Button>
        </form>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Members</CardTitle>
        <CardDescription>People with access to this organization.</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <Button variant="outline" size="sm" type="button" class="cursor-pointer" @click="inviteOpen = true">
          Add member
        </Button>

        <div v-if="organizationStore.membersLoading" class="text-sm text-muted-foreground">Loading…</div>

        <div v-if="!organizationStore.membersLoading && organizationStore.members.length > 0" class="overflow-hidden border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Email</TableHead>
                <TableHead class="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="member in organizationStore.members" :key="member.id">
                <TableCell class="max-w-[200px] font-medium">
                  <span class="block truncate">{{ member.name }}</span>
                </TableCell>
                <TableCell class="max-w-[280px] text-muted-foreground">
                  <span class="block truncate">{{ member.email }}</span>
                </TableCell>
                <TableCell class="text-right">
                  <Button
                    v-if="member.id !== authStore.user?.id"
                    variant="ghost"
                    size="sm"
                    type="button"
                    class="text-destructive hover:text-destructive cursor-pointer"
                    @click="openRemove(member.id)"
                  >
                    Remove
                  </Button>
                  <span v-else class="text-xs text-muted-foreground">You</span>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>

        <p v-if="!organizationStore.membersLoading && organizationStore.members.length === 0" class="text-sm text-muted-foreground">
          No other members yet. Invite someone by email.
        </p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-destructive">Danger zone</CardTitle>
        <CardDescription>
          Permanently delete this organization and related data. This cannot be undone.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Button variant="destructive" type="button" class="cursor-pointer" @click="deleteOpen = true">
          Delete organization
        </Button>
      </CardContent>
    </Card>

    <Dialog v-model:open="inviteOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add member</DialogTitle>
          <DialogDescription>Enter the email address of an existing user account.</DialogDescription>
        </DialogHeader>
        <form @submit.prevent="submitInvite">
          <div class="grid gap-2 py-2">
            <Label for="member-invitation-email">Email</Label>
            <Input
              id="member-invitation-email"
              type="email"
              v-model="inviteEmail"
              autocomplete="off"
              placeholder="colleague@example.com"
            />
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="inviteOpen = false">Cancel</Button>
            <Button type="submit" class="cursor-pointer" :disabled="inviteSubmitting || !inviteEmail.trim()">
              <Loader2 v-if="inviteSubmitting" class="mr-1.5 size-3.5 animate-spin" />
              Add
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <AlertDialog v-model:open="removeOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Remove member?</AlertDialogTitle>
          <AlertDialogDescription>This user will lose access to this organization.</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="removeSubmitting">Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90 cursor-pointer"
            :disabled="removeSubmitting"
            @click="confirmRemove"
          >
            <Loader2 v-if="removeSubmitting" class="mr-1.5 size-3.5 animate-spin" />
            Remove
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <AlertDialog v-model:open="deleteOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete this organization?</AlertDialogTitle>
          <AlertDialogDescription>
            All projects and data in this organization will be removed. This action cannot be undone.
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
            Delete organization
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
