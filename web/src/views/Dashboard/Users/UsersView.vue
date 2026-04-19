<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useTitle } from '@vueuse/core'
import { Ellipsis, Loader2 } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth.store'
import { useUsersStore } from '@/stores/users.store'
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
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

useTitle('Users — neond')
const authStore = useAuthStore()
const usersStore = useUsersStore()

const createOpen = ref(false)
const createName = ref('')
const createEmail = ref('')
const createPassword = ref('')
const createSubmitting = ref(false)

const editOpen = ref(false)
const editUserId = ref('')
const editName = ref('')
const editEmail = ref('')
const editIsAdmin = ref(false)
const editSubmitting = ref(false)

const resetPasswordOpen = ref(false)
const resetPasswordUserId = ref('')
const resetPasswordValue = ref('')
const resetPasswordSubmitting = ref(false)

const deleteOpen = ref(false)
const deleteUserId = ref('')
const deleteSubmitting = ref(false)

onMounted(() => {
  usersStore.fetch()
})

function openCreate() {
  createName.value = ''
  createEmail.value = ''
  createPassword.value = ''
  createOpen.value = true
}

async function submitCreate() {
  createSubmitting.value = true
  await usersStore.create(createName.value, createEmail.value, createPassword.value)
  createOpen.value = false
  createSubmitting.value = false
}

function openEdit(userId: string) {
  const user = usersStore.users.find((u) => u.id === userId)
  if (!user) return
  editUserId.value = user.id
  editName.value = user.name
  editEmail.value = user.email
  editIsAdmin.value = user.is_admin
  editOpen.value = true
}

async function submitEdit() {
  editSubmitting.value = true
  await usersStore.update(editUserId.value, {
    name: editName.value,
    email: editEmail.value,
    is_admin: editIsAdmin.value,
  })
  await authStore.refreshUser()
  editOpen.value = false
  editSubmitting.value = false
}

function openResetPassword(userId: string) {
  resetPasswordUserId.value = userId
  resetPasswordValue.value = ''
  resetPasswordOpen.value = true
}

async function submitResetPassword() {
  resetPasswordSubmitting.value = true
  await usersStore.resetPassword(resetPasswordUserId.value, resetPasswordValue.value)
  resetPasswordOpen.value = false
  resetPasswordSubmitting.value = false
}

function openDelete(userId: string) {
  deleteUserId.value = userId
  deleteOpen.value = true
}

async function confirmDelete() {
  deleteSubmitting.value = true
  await usersStore.remove(deleteUserId.value)
  deleteOpen.value = false
  deleteSubmitting.value = false
}
</script>

<template>
  <div class="w-full divide-y">
    <section class="flex flex-col gap-4 pb-8">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-sm font-medium">Users</h3>
          <p class="mt-1 text-sm text-muted-foreground">Manage user accounts for this instance.</p>
        </div>
        <Button size="sm" type="button" class="cursor-pointer" @click="openCreate">
          Create user
        </Button>
      </div>

      <div v-if="usersStore.loading" class="text-sm text-muted-foreground">Loading…</div>

      <div v-if="!usersStore.loading && usersStore.users.length > 0" class="overflow-hidden border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Email</TableHead>
              <TableHead>Role</TableHead>
              <TableHead class="text-right w-16"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="user in usersStore.users" :key="user.id">
              <TableCell class="max-w-50 font-medium">
                <span class="block truncate">{{ user.name }}</span>
              </TableCell>
              <TableCell class="max-w-70 text-muted-foreground">
                <span class="block truncate">{{ user.email }}</span>
              </TableCell>
              <TableCell>
                <Badge v-if="user.is_admin" variant="default">Admin</Badge>
                <Badge v-else variant="secondary">User</Badge>
              </TableCell>
              <TableCell class="text-right">
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon" class="size-8 cursor-pointer">
                      <Ellipsis class="size-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem @click="openEdit(user.id)">Edit</DropdownMenuItem>
                    <DropdownMenuItem @click="openResetPassword(user.id)">Reset password</DropdownMenuItem>
                    <template v-if="user.id !== authStore.user?.id">
                      <DropdownMenuSeparator />
                      <DropdownMenuItem class="text-destructive" @click="openDelete(user.id)">Delete</DropdownMenuItem>
                    </template>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </section>

    <Dialog v-model:open="createOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create user</DialogTitle>
          <DialogDescription>Create a new user account with email and password.</DialogDescription>
        </DialogHeader>
        <form @submit.prevent="submitCreate">
          <div class="grid gap-4 py-2">
            <div class="grid gap-2">
              <Label for="create-name">Name</Label>
              <Input id="create-name" v-model="createName" required />
            </div>
            <div class="grid gap-2">
              <Label for="create-email">Email</Label>
              <Input id="create-email" type="email" v-model="createEmail" required />
            </div>
            <div class="grid gap-2">
              <Label for="create-password">Password</Label>
              <Input id="create-password" type="password" v-model="createPassword" required autocomplete="new-password" />
            </div>
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="createOpen = false">Cancel</Button>
            <Button type="submit" class="cursor-pointer" :disabled="createSubmitting || !createName.trim() || !createEmail.trim() || !createPassword">
              <Loader2 v-if="createSubmitting" class="mr-1.5 size-3.5 animate-spin" />
              Create
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="editOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Edit user</DialogTitle>
          <DialogDescription>Update user details and role.</DialogDescription>
        </DialogHeader>
        <form @submit.prevent="submitEdit">
          <div class="grid gap-4 py-2">
            <div class="grid gap-2">
              <Label for="edit-name">Name</Label>
              <Input id="edit-name" v-model="editName" required />
            </div>
            <div class="grid gap-2">
              <Label for="edit-email">Email</Label>
              <Input id="edit-email" type="email" v-model="editEmail" required />
            </div>
            <div class="flex items-center gap-2">
              <Checkbox
                id="edit-admin"
                v-model="editIsAdmin"
                :disabled="editUserId === authStore.user?.id"
              />
              <Label for="edit-admin">Administrator</Label>
            </div>
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="editOpen = false">Cancel</Button>
            <Button type="submit" class="cursor-pointer" :disabled="editSubmitting || !editName.trim() || !editEmail.trim()">
              <Loader2 v-if="editSubmitting" class="mr-1.5 size-3.5 animate-spin" />
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="resetPasswordOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Reset password</DialogTitle>
          <DialogDescription>Set a new password for this user.</DialogDescription>
        </DialogHeader>
        <form @submit.prevent="submitResetPassword">
          <div class="grid gap-4 py-2">
            <div class="grid gap-2">
              <Label for="reset-password">New password</Label>
              <Input id="reset-password" type="password" v-model="resetPasswordValue" required autocomplete="new-password" />
            </div>
          </div>
          <DialogFooter class="mt-2">
            <Button variant="outline" type="button" class="cursor-pointer" @click="resetPasswordOpen = false">Cancel</Button>
            <Button type="submit" class="cursor-pointer" :disabled="resetPasswordSubmitting || !resetPasswordValue">
              <Loader2 v-if="resetPasswordSubmitting" class="mr-1.5 size-3.5 animate-spin" />
              Reset
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <AlertDialog v-model:open="deleteOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete user?</AlertDialogTitle>
          <AlertDialogDescription>This user will be permanently deleted. This action cannot be undone.</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="deleteSubmitting">Cancel</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90 cursor-pointer"
            :disabled="deleteSubmitting"
            @click="confirmDelete"
          >
            <Loader2 v-if="deleteSubmitting" class="mr-1.5 size-3.5 animate-spin" />
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
