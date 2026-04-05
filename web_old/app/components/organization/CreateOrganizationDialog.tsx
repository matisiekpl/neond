import * as React from "react"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import { useOrganizationStore } from "~/stores/organization-store"
import { getAppError } from "~/lib/errors"
import { Button } from "~/components/ui/button"
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

type CreateOrganizationDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
}

type FormFields = {
  name: string
}

export function CreateOrganizationDialog({
  open,
  onOpenChange,
}: CreateOrganizationDialogProps) {
  const createOrganization = useOrganizationStore((s) => s.createOrganization)
  const { register, handleSubmit, reset, watch, formState: { isSubmitting } } = useForm<FormFields>({
    defaultValues: { name: "" },
  })

  React.useEffect(() => {
    if (!open) reset()
  }, [open, reset])

  async function onSubmit({ name }: FormFields) {
    const trimmed = name.trim()
    if (!trimmed) return
    try {
      await createOrganization(trimmed)
      onOpenChange(false)
    } catch (e) {
      toast.error(getAppError(e))
    }
  }

  const name = watch("name")

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create organization</DialogTitle>
          <DialogDescription>
            Add a new organization. You will be added as a member automatically.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)}>
          <div className="grid gap-2">
            <Label htmlFor="create-org-name">Name</Label>
            <Input
              id="create-org-name"
              autoFocus
              {...register("name")}
              placeholder="Organization name"
            />
          </div>
          <DialogFooter className="mt-4">
            <Button variant="outline" type="button" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={isSubmitting || !name.trim()}
            >
              Create
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}