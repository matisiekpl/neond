<script setup lang="ts">
import {Button} from '@/components/ui/button'
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from '@/components/ui/card'
import {Input} from '@/components/ui/input'
import {Label} from '@/components/ui/label'
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {useOrganizationStore} from '@/stores/organization.store.ts'
import {useAuthStore} from '@/stores/auth.store.ts'
import {getAppError} from '@/api/utils.ts'
import {computed, onMounted, ref, watch} from 'vue'
import {useTitle} from '@vueuse/core'
import {toast} from 'vue-sonner'

useTitle('Organization settings — neond')

const organizationStore = useOrganizationStore()
const authStore = useAuthStore()

const organizationName = ref('')
const isSavingOrganizationName = ref(false)

const memberInvitationEmail = ref('')
const memberInvitationDialogOpen = ref(false)
const isMemberInvitationSubmitting = ref(false)

const deleteOrganizationDialogOpen = ref(false)
const isDeletingOrganization = ref(false)

const memberRemovalTargetUserId = ref<string | null>(null)
const memberRemovalDialogOpen = ref(false)
const isMemberRemovalSubmitting = ref(false)

const selectedOrganizationId = computed(() => organizationStore.selectedOrganizationId)

watch(
  () => organizationStore.currentOrganization,
  (organization) => {
    if (organization) organizationName.value = organization.name
  },
  {immediate: true},
)

onMounted(async () => {
  if (selectedOrganizationId.value) {
    await organizationStore.fetchMembers(selectedOrganizationId.value)
  }
})

watch(selectedOrganizationId, async (organizationId) => {
  if (organizationId) {
    await organizationStore.fetchMembers(organizationId)
  } else {
    organizationStore.members = []
  }
})

async function saveOrganizationName() {
  if (!selectedOrganizationId.value) return
  const trimmedOrganizationName = organizationName.value.trim()
  if (!trimmedOrganizationName) return
  isSavingOrganizationName.value = true
  try {
    await organizationStore.updateOrganization(selectedOrganizationId.value, trimmedOrganizationName)
  } catch (error) {
    toast.error(getAppError(error))
  } finally {
    isSavingOrganizationName.value = false
  }
}

async function submitMemberInvitation() {
  if (!selectedOrganizationId.value) return
  const trimmedMemberInvitationEmail = memberInvitationEmail.value.trim()
  if (!trimmedMemberInvitationEmail) return
  isMemberInvitationSubmitting.value = true
  try {
    await organizationStore.addMemberByEmail(selectedOrganizationId.value, trimmedMemberInvitationEmail)
    memberInvitationDialogOpen.value = false
    memberInvitationEmail.value = ''
  } catch {
  } finally {
    isMemberInvitationSubmitting.value = false
  }
}

async function confirmMemberRemoval() {
  if (!selectedOrganizationId.value || !memberRemovalTargetUserId.value) return
  isMemberRemovalSubmitting.value = true
  try {
    await organizationStore.removeMember(selectedOrganizationId.value, memberRemovalTargetUserId.value)
    memberRemovalDialogOpen.value = false
    memberRemovalTargetUserId.value = null
  } catch {
  } finally {
    isMemberRemovalSubmitting.value = false
  }
}

async function confirmOrganizationDeletion() {
  if (!selectedOrganizationId.value || !authStore.user) return
  isDeletingOrganization.value = true
  try {
    await organizationStore.deleteOrganization(selectedOrganizationId.value)
    deleteOrganizationDialogOpen.value = false
    await organizationStore.ensureOrganizations(authStore.user.name)
  } catch (error) {
    toast.error(getAppError(error))
  } finally {
    isDeletingOrganization.value = false
  }
}

function openMemberRemovalDialog(userId: string) {
  memberRemovalTargetUserId.value = userId
  memberRemovalDialogOpen.value = true
}
</script>

<template>
  <section
    v-if="!organizationStore.currentOrganization"
    class="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center"
  >
    <p class="text-sm text-muted-foreground">No organization selected.</p>
  </section>

  <section v-else class="mx-auto w-full max-w-3xl space-y-6">
    <Card>
      <CardHeader>
        <CardTitle>General</CardTitle>
        <CardDescription>Update your organization display name.</CardDescription>
      </CardHeader>
      <CardContent class="max-w-lg space-y-4">
        <div class="space-y-2">
          <Label for="organization-name">Name</Label>
          <Input id="organization-name" v-model="organizationName" class="w-full" />
        </div>
        <Button
          type="button"
          :disabled="isSavingOrganizationName || !organizationName.trim()"
          @click="saveOrganizationName"
        >
          Save changes
        </Button>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Members</CardTitle>
        <CardDescription>People with access to this organization.</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <Button variant="outline" size="sm" type="button" @click="memberInvitationDialogOpen = true">
          Add member
        </Button>
        <div v-if="organizationStore.membersLoading" class="text-sm text-muted-foreground">Loading…</div>
        <ul v-else-if="organizationStore.members.length > 0" class="divide-y">
          <li
            v-for="member in organizationStore.members"
            :key="member.id"
            class="flex items-center justify-between gap-4 py-3 text-sm"
          >
            <div class="min-w-0 flex-1">
              <div class="truncate font-medium">{{ member.name }}</div>
              <div class="truncate text-muted-foreground">{{ member.email }}</div>
            </div>
            <Button
              v-if="member.id !== authStore.user?.id"
              variant="ghost"
              size="sm"
              class="shrink-0 text-destructive hover:text-destructive"
              type="button"
              @click="openMemberRemovalDialog(member.id)"
            >
              Remove
            </Button>
            <span v-else class="shrink-0 text-xs text-muted-foreground">You</span>
          </li>
        </ul>
        <p v-else class="text-sm text-muted-foreground">No other members yet. Invite someone by email.</p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-destructive">Danger zone</CardTitle>
        <CardDescription>
          Permanently delete this organization and related data. This cannot be undone.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <p class="text-sm text-muted-foreground">
          You will lose access to all projects in this organization.
        </p>
        <Button variant="destructive" type="button" @click="deleteOrganizationDialogOpen = true">
          Delete organization
        </Button>
      </CardContent>
    </Card>
  </section>

  <Dialog v-model:open="memberInvitationDialogOpen">
    <DialogContent class="sm:max-w-md" @open-auto-focus.prevent>
      <DialogHeader>
        <DialogTitle>Add member</DialogTitle>
        <DialogDescription>Enter the email address of an existing user account.</DialogDescription>
      </DialogHeader>
      <div class="grid gap-2 py-2">
        <Label for="member-invitation-email">Email</Label>
        <Input
          id="member-invitation-email"
          v-model="memberInvitationEmail"
          type="email"
          autocomplete="off"
          placeholder="colleague@example.com"
          @keydown.enter.prevent="submitMemberInvitation"
        />
      </div>
      <DialogFooter>
        <Button variant="outline" type="button" @click="memberInvitationDialogOpen = false">Cancel</Button>
        <Button
          type="button"
          :disabled="isMemberInvitationSubmitting || !memberInvitationEmail.trim()"
          @click="submitMemberInvitation"
        >
          Add
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <AlertDialog v-model:open="memberRemovalDialogOpen">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Remove member?</AlertDialogTitle>
        <AlertDialogDescription>This user will lose access to this organization.</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel :disabled="isMemberRemovalSubmitting">Cancel</AlertDialogCancel>
        <AlertDialogAction
          class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          :disabled="isMemberRemovalSubmitting"
          @click="confirmMemberRemoval"
        >
          Remove
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>

  <AlertDialog v-model:open="deleteOrganizationDialogOpen">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Delete this organization?</AlertDialogTitle>
        <AlertDialogDescription>
          All projects and data in this organization will be removed. This action cannot be undone.
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel :disabled="isDeletingOrganization">Cancel</AlertDialogCancel>
        <AlertDialogAction
          class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          :disabled="isDeletingOrganization"
          @click="confirmOrganizationDeletion"
        >
          Delete organization
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
