import * as React from "react"
import { toast } from "sonner"
import { useProjectStore } from "~/stores/project-store"
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

const PG_VERSIONS = ["V17", "V16", "V15", "V14"] as const

type CreateProjectDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateProjectDialog({ open, onOpenChange }: CreateProjectDialogProps) {
  const createProject = useProjectStore((s) => s.createProject)
  const selectedOrganizationId = useOrganizationStore((s) => s.selectedOrganizationId)
  const [name, setName] = React.useState("")
  const [pgVersion, setPgVersion] = React.useState<string>("V17")
  const [creating, setCreating] = React.useState(false)

  React.useEffect(() => {
    if (!open) {
      setName("")
      setPgVersion("V17")
    }
  }, [open])

  async function submit() {
    const trimmed = name.trim()
    if (!trimmed || !selectedOrganizationId) return
    setCreating(true)
    try {
      await createProject(selectedOrganizationId, { name: trimmed, pg_version: pgVersion })
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
          <DialogTitle>Create project</DialogTitle>
          <DialogDescription>
            A project contains branches and compute endpoints.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          <div className="grid gap-2">
            <Label htmlFor="create-project-name">Name</Label>
            <Input
              id="create-project-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="my-project"
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault()
                  void submit()
                }
              }}
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="create-project-pg">PostgreSQL version</Label>
            <select
              id="create-project-pg"
              value={pgVersion}
              onChange={(e) => setPgVersion(e.target.value)}
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
