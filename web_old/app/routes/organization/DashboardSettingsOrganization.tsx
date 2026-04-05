import * as React from "react"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import { useShallow } from "zustand/react/shallow"
import { useAuthStore } from "~/stores/auth-store"
import { useOrganizationStore } from "~/stores/organization-store"
import { getAppError } from "~/lib/errors"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "~/components/ui/alert-dialog"
import { Loader2 } from "lucide-react"
import { Button } from "~/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "~/components/ui/dialog"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "~/components/ui/table"

type OrganizationNameFields = { organizationName: string }
type InviteFields = { inviteEmail: string }

export default function OrganizationSettingsRoute() {
  const { user } = useAuthStore(useShallow((s) => ({ user: s.user })))
  const {
    currentOrganization,
    selectedOrganizationId,
    members,
    membersLoading,
    updateOrganization,
    addMemberByEmail,
    removeMember,
    deleteOrganization,
    fetchMembers,
    loadOrganizations,
  } = useOrganizationStore(
    useShallow((s) => ({
      currentOrganization: s.selectedOrganizationId
        ? s.organizations.find((o) => o.id === s.selectedOrganizationId)
        : undefined,
      selectedOrganizationId: s.selectedOrganizationId,
      members: s.members,
      membersLoading: s.membersLoading,
      updateOrganization: s.updateOrganization,
      addMemberByEmail: s.addMemberByEmail,
      removeMember: s.removeMember,
      deleteOrganization: s.deleteOrganization,
      fetchMembers: s.fetchMembers,
      loadOrganizations: s.loadOrganizations,
    })),
  )

  const nameForm = useForm<OrganizationNameFields>({
    defaultValues: { organizationName: "" },
  })
  const inviteForm = useForm<InviteFields>({
    defaultValues: { inviteEmail: "" },
  })

  const [inviteOpen, setInviteOpen] = React.useState(false)
  const [removeOpen, setRemoveOpen] = React.useState(false)
  const [removeUserId, setRemoveUserId] = React.useState<string | null>(null)
  const [removeSubmitting, setRemoveSubmitting] = React.useState(false)
  const [deleteOpen, setDeleteOpen] = React.useState(false)
  const [deleteSubmitting, setDeleteSubmitting] = React.useState(false)

  React.useEffect(() => {
    document.title = "Organization settings — neond"
  }, [])

  React.useEffect(() => {
    if (currentOrganization) {
      nameForm.reset({ organizationName: currentOrganization.name })
    }
  }, [currentOrganization])

  React.useEffect(() => {
    if (selectedOrganizationId) {
      void fetchMembers(selectedOrganizationId)
    }
  }, [selectedOrganizationId, fetchMembers])

  React.useEffect(() => {
    if (!inviteOpen) inviteForm.reset()
  }, [inviteOpen])

  async function saveOrganizationName({ organizationName }: OrganizationNameFields) {
    if (!selectedOrganizationId) return
    const trimmed = organizationName.trim()
    if (!trimmed) return
    try {
      await updateOrganization(selectedOrganizationId, trimmed)
      nameForm.reset({ organizationName: trimmed })
    } catch (err) {
      toast.error(getAppError(err))
    }
  }

  async function submitInvite({ inviteEmail }: InviteFields) {
    if (!selectedOrganizationId) return
    const trimmed = inviteEmail.trim()
    if (!trimmed) return
    try {
      await addMemberByEmail(selectedOrganizationId, trimmed)
      setInviteOpen(false)
    } catch {
    }
  }

  async function confirmRemove() {
    if (!selectedOrganizationId || !removeUserId) return
    setRemoveSubmitting(true)
    try {
      await removeMember(selectedOrganizationId, removeUserId)
      setRemoveOpen(false)
      setRemoveUserId(null)
    } catch {
    } finally {
      setRemoveSubmitting(false)
    }
  }

  async function confirmDelete() {
    if (!selectedOrganizationId || !user) return
    setDeleteSubmitting(true)
    try {
      await deleteOrganization(selectedOrganizationId)
      setDeleteOpen(false)
      await loadOrganizations()
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      setDeleteSubmitting(false)
    }
  }

  function openRemove(userId: string) {
    setRemoveUserId(userId)
    setRemoveOpen(true)
  }

  if (!currentOrganization) {
    return (
      <section className="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center">
        <p className="text-sm text-muted-foreground">No organization selected.</p>
      </section>
    )
  }

  const watchedName = nameForm.watch("organizationName")
  const watchedEmail = inviteForm.watch("inviteEmail")

  return (
    <div className="w-full space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>General</CardTitle>
          <CardDescription>
            Update your organization display name.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <form onSubmit={nameForm.handleSubmit(saveOrganizationName)}>
            <div className="space-y-2">
              <Label htmlFor="organization-name">Name</Label>
              <Input
                id="organization-name"
                {...nameForm.register("organizationName")}
                className="w-full"
              />
            </div>
            <Button
              type="submit"
              className="mt-4"
              disabled={nameForm.formState.isSubmitting || !watchedName.trim() || !nameForm.formState.isDirty}
            >
              {nameForm.formState.isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Save changes
            </Button>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Members</CardTitle>
          <CardDescription>
            People with access to this organization.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            variant="outline"
            size="sm"
            type="button"
            onClick={() => setInviteOpen(true)}
          >
            Add member
          </Button>
          {membersLoading ? (
            <div className="text-sm text-muted-foreground">Loading…</div>
          ) : null}
          {!membersLoading && members.length > 0 ? (
            <div className="overflow-hidden border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {members.map((member) => (
                    <TableRow key={member.id}>
                      <TableCell className="max-w-[200px] font-medium">
                        <span className="block truncate">{member.name}</span>
                      </TableCell>
                      <TableCell className="max-w-[280px] text-muted-foreground">
                        <span className="block truncate">{member.email}</span>
                      </TableCell>
                      <TableCell className="text-right">
                        {member.id !== user?.id ? (
                          <Button
                            variant="ghost"
                            size="sm"
                            type="button"
                            className="text-destructive hover:text-destructive"
                            onClick={() => openRemove(member.id)}
                          >
                            Remove
                          </Button>
                        ) : (
                          <span className="text-xs text-muted-foreground">
                            You
                          </span>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          ) : null}
          {!membersLoading && members.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No other members yet. Invite someone by email.
            </p>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-destructive">Danger zone</CardTitle>
          <CardDescription>
            Permanently delete this organization and related data. This cannot
            be undone.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            variant="destructive"
            type="button"
            onClick={() => setDeleteOpen(true)}
          >
            Delete organization
          </Button>
        </CardContent>
      </Card>

      <Dialog open={inviteOpen} onOpenChange={setInviteOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Add member</DialogTitle>
            <DialogDescription>
              Enter the email address of an existing user account.
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={inviteForm.handleSubmit(submitInvite)}>
            <div className="grid gap-2 py-2">
              <Label htmlFor="member-invitation-email">Email</Label>
              <Input
                id="member-invitation-email"
                type="email"
                {...inviteForm.register("inviteEmail")}
                autoComplete="off"
                placeholder="colleague@example.com"
              />
            </div>
            <DialogFooter className="mt-2">
              <Button
                variant="outline"
                type="button"
                onClick={() => setInviteOpen(false)}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={inviteForm.formState.isSubmitting || !watchedEmail.trim()}
              >
                {inviteForm.formState.isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
                Add
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <AlertDialog open={removeOpen} onOpenChange={setRemoveOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove member?</AlertDialogTitle>
            <AlertDialogDescription>
              This user will lose access to this organization.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={removeSubmitting}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              disabled={removeSubmitting}
              onClick={() => void confirmRemove()}
            >
              {removeSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete this organization?</AlertDialogTitle>
            <AlertDialogDescription>
              All projects and data in this organization will be removed. This
              action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleteSubmitting}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              disabled={deleteSubmitting}
              onClick={() => void confirmDelete()}
            >
              {deleteSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Delete organization
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
