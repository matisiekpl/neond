import * as React from "react"
import { Link } from "react-router"
import { toast } from "sonner"
import { useShallow } from "zustand/react/shallow"
import { MoreHorizontal } from "lucide-react"
import { useProjectStore } from "~/stores/project-store"
import { useOrganizationStore } from "~/stores/organization-store"
import { getAppError } from "~/lib/errors"
import { CreateProjectDialog } from "~/components/projects/CreateProjectDialog"
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
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "~/components/ui/dialog"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { Spinner } from "~/components/ui/spinner"

export default function ProjectsIndexRoute() {
  const selectedOrganizationId = useOrganizationStore((s) => s.selectedOrganizationId)
  const { projects, loading, fetchProjects, updateProject, deleteProject } =
    useProjectStore(
      useShallow((s) => ({
        projects: s.projects,
        loading: s.loading,
        fetchProjects: s.fetchProjects,
        updateProject: s.updateProject,
        deleteProject: s.deleteProject,
      })),
    )

  const [createOpen, setCreateOpen] = React.useState(false)

  const [renameOpen, setRenameOpen] = React.useState(false)
  const [renameId, setRenameId] = React.useState<string | null>(null)
  const [renameName, setRenameName] = React.useState("")
  const [renameSubmitting, setRenameSubmitting] = React.useState(false)

  const [deleteOpen, setDeleteOpen] = React.useState(false)
  const [deleteId, setDeleteId] = React.useState<string | null>(null)
  const [deleteSubmitting, setDeleteSubmitting] = React.useState(false)

  React.useEffect(() => {
    document.title = "Projects — neond"
  }, [])

  React.useEffect(() => {
    if (selectedOrganizationId) {
      void fetchProjects(selectedOrganizationId)
    }
  }, [selectedOrganizationId, fetchProjects])

  function openRename(id: string, currentName: string) {
    setRenameId(id)
    setRenameName(currentName)
    setRenameOpen(true)
  }

  function openDelete(id: string) {
    setDeleteId(id)
    setDeleteOpen(true)
  }

  async function submitRename() {
    if (!selectedOrganizationId || !renameId) return
    const trimmed = renameName.trim()
    if (!trimmed) return
    setRenameSubmitting(true)
    try {
      await updateProject(selectedOrganizationId, renameId, { name: trimmed })
      setRenameOpen(false)
    } catch (e) {
      toast.error(getAppError(e))
    } finally {
      setRenameSubmitting(false)
    }
  }

  async function confirmDelete() {
    if (!selectedOrganizationId || !deleteId) return
    setDeleteSubmitting(true)
    try {
      await deleteProject(selectedOrganizationId, deleteId)
      setDeleteOpen(false)
    } catch {
    } finally {
      setDeleteSubmitting(false)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold">Projects</h1>
          <p className="text-sm text-muted-foreground">
            Manage your database projects.
          </p>
        </div>
        <Button type="button" onClick={() => setCreateOpen(true)}>
          New project
        </Button>
      </div>

      {loading ? (
        <div className="flex justify-center py-12">
          <Spinner className="size-6" />
        </div>
      ) : projects.length === 0 ? (
        <section className="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center">
          <p className="text-sm font-medium">No projects yet</p>
          <p className="mt-1 text-sm text-muted-foreground">
            Create your first project to get started.
          </p>
          <Button
            type="button"
            className="mt-4"
            onClick={() => setCreateOpen(true)}
          >
            New project
          </Button>
        </section>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {projects.map((project) => (
            <div key={project.id} className="relative">
              <Link to={`/dashboard/projects/${project.id}`} className="block">
                <Card className="transition-colors hover:bg-accent/50">
                  <CardHeader className="pb-2 pr-10">
                    <CardTitle className="truncate text-base">
                      {project.name}
                    </CardTitle>
                    <CardDescription>
                      PostgreSQL {project.pg_version.replace(/^V/i, "")}
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <p className="text-xs text-muted-foreground">
                      Created{" "}
                      {new Date(project.created_at).toLocaleDateString(undefined, {
                        year: "numeric",
                        month: "short",
                        day: "numeric",
                      })}
                    </p>
                  </CardContent>
                </Card>
              </Link>
              <div className="absolute right-3 top-3">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="size-7"
                      onClick={(e) => e.preventDefault()}
                    >
                      <MoreHorizontal className="size-4" />
                      <span className="sr-only">Open menu</span>
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem
                      onClick={() => openRename(project.id, project.name)}
                    >
                      Rename
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      className="text-destructive focus:text-destructive"
                      onClick={() => openDelete(project.id)}
                    >
                      Delete
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
          ))}
        </div>
      )}

      <CreateProjectDialog open={createOpen} onOpenChange={setCreateOpen} />

      <Dialog open={renameOpen} onOpenChange={setRenameOpen}>
        <DialogContent className="sm:max-w-md" onOpenAutoFocus={(e) => e.preventDefault()}>
          <DialogHeader>
            <DialogTitle>Rename project</DialogTitle>
          </DialogHeader>
          <div className="grid gap-2 py-2">
            <Label htmlFor="rename-project-name">Name</Label>
            <Input
              id="rename-project-name"
              value={renameName}
              onChange={(e) => setRenameName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault()
                  void submitRename()
                }
              }}
            />
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              type="button"
              onClick={() => setRenameOpen(false)}
            >
              Cancel
            </Button>
            <Button
              type="button"
              disabled={renameSubmitting || !renameName.trim()}
              onClick={() => void submitRename()}
            >
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete project?</AlertDialogTitle>
            <AlertDialogDescription>
              All branches and data in this project will be permanently removed.
              This action cannot be undone.
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
              Delete project
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
