import * as React from "react"
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

export function CreateOrganizationDialog({
  open,
  onOpenChange,
}: CreateOrganizationDialogProps) {
  const createOrganization = useOrganizationStore((s) => s.createOrganization)
  const [name, setName] = React.useState("")
  const [creating, setCreating] = React.useState(false)

  React.useEffect(() => {
    if (!open) {
      setName("")
    }
  }, [open])

  async function submit() {
    const trimmed = name.trim()
    if (!trimmed) {
      return
    }
    setCreating(true)
    try {
      await createOrganization(trimmed)
      onOpenChange(false)
    } catch (e) {
      toast.error(getAppError(e))
    } finally {
      setCreating(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create organization</DialogTitle>
          <DialogDescription>
            Add a new organization. You will be added as a member automatically.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-2">
          <Label htmlFor="create-org-name">Name</Label>
          <Input
            id="create-org-name"
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Organization name"
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault()
                void submit()
              }
            }}
          />
        </div>
        <DialogFooter>
          <Button variant="outline" type="button" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            type="button"
            disabled={creating || !name.trim()}
            onClick={() => void submit()}
          >
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
