import * as React from "react"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import { useProjectStore } from "~/stores/project-store"
import { useOrganizationStore } from "~/stores/organization-store"
import { getAppError } from "~/lib/errors"
import { Loader2 } from "lucide-react"
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

const PG_VERSIONS = ["V17", "V16", "V15", "V14"] as const

type CreateProjectDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
}

type FormFields = {
  name: string
  pgVersion: string
}

export function CreateProjectDialog({ open, onOpenChange }: CreateProjectDialogProps) {
  const createProject = useProjectStore((s) => s.createProject)
  const selectedOrganizationId = useOrganizationStore((s) => s.selectedOrganizationId)
  const { register, handleSubmit, reset, watch, formState: { isSubmitting } } = useForm<FormFields>({
    defaultValues: { name: "", pgVersion: "V17" },
  })

  React.useEffect(() => {
    if (!open) reset()
  }, [open, reset])

  async function onSubmit({ name, pgVersion }: FormFields) {
    const trimmed = name.trim()
    if (!trimmed || !selectedOrganizationId) return
    try {
      await createProject(selectedOrganizationId, { name: trimmed, pg_version: pgVersion })
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
          <DialogTitle>Create project</DialogTitle>
          <DialogDescription>
            A project contains branches and compute endpoints.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)}>
          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="create-project-name">Name</Label>
              <Input
                id="create-project-name"
                autoFocus
                {...register("name")}
                placeholder="my-project"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="create-project-pg">PostgreSQL version</Label>
              <select
                id="create-project-pg"
                {...register("pgVersion")}
                className="flex h-9 w-full border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              >
                {PG_VERSIONS.map((v) => (
                  <option key={v} value={v}>
                    PostgreSQL {v.slice(1)}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <DialogFooter className="mt-4">
            <Button variant="outline" type="button" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={isSubmitting || !name.trim()}
            >
              {isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Create
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}