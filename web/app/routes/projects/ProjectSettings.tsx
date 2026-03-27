import * as React from "react"
import { useParams, useNavigate } from "react-router"
import { useShallow } from "zustand/react/shallow"
import { useProjectStore } from "~/stores/project-store"
import { useOrganizationStore } from "~/stores/organization-store"
import { getAppError } from "~/lib/errors"
import { toast } from "sonner"
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
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { Spinner } from "~/components/ui/spinner"

export default function ProjectSettingsRoute() {
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()

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

  const project = projects.find((p) => p.id === projectId)

  const [name, setName] = React.useState("")
  const [savingName, setSavingName] = React.useState(false)
  const [deleteOpen, setDeleteOpen] = React.useState(false)
  const [deleting, setDeleting] = React.useState(false)

  React.useEffect(() => {
    if (selectedOrganizationId) {
      void fetchProjects(selectedOrganizationId)
    }
  }, [selectedOrganizationId, fetchProjects])

  React.useEffect(() => {
    if (project) {
      setName(project.name)
      document.title = `Settings — ${project.name} — neond`
    }
  }, [project?.id])

  async function saveName() {
    if (!selectedOrganizationId || !projectId) return
    const trimmed = name.trim()
    if (!trimmed || trimmed === project?.name) return
    setSavingName(true)
    try {
      await updateProject(selectedOrganizationId, projectId, trimmed)
    } catch (e) {
      toast.error(getAppError(e))
    } finally {
      setSavingName(false)
    }
  }

  async function confirmDelete() {
    if (!selectedOrganizationId || !projectId) return
    setDeleting(true)
    try {
      await deleteProject(selectedOrganizationId, projectId)
      navigate("/dashboard/projects")
    } catch {
      setDeleting(false)
    }
  }

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <Spinner className="size-6" />
      </div>
    )
  }

  if (!project) {
    return (
      <div className="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center">
        <p className="text-sm font-medium">Project not found</p>
        <button
          type="button"
          className="mt-4 text-sm underline underline-offset-4"
          onClick={() => navigate("/dashboard/projects")}
        >
          Back to projects
        </button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-lg font-semibold">Project settings</h1>
        <p className="text-sm text-muted-foreground">Manage your project configuration.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>General</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-2">
            <Label htmlFor="project-name">Name</Label>
            <Input
              id="project-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault()
                  void saveName()
                }
              }}
            />
          </div>
          <Button
            type="button"
            disabled={savingName || !name.trim() || name.trim() === project.name}
            onClick={() => void saveName()}
          >
            Save changes
          </Button>
        </CardContent>
      </Card>

      <Card className="border-destructive">
        <CardHeader>
          <CardTitle className="text-destructive">Danger zone</CardTitle>
          <CardDescription>
            Irreversible actions that affect this project.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-xs font-medium">Delete project</p>
              <p className="text-xs text-muted-foreground">
                Permanently remove this project and all its data.
              </p>
            </div>
            <Button
              variant="destructive"
              type="button"
              onClick={() => setDeleteOpen(true)}
            >
              Delete project
            </Button>
          </div>
        </CardContent>
      </Card>

      <AlertDialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete project?</AlertDialogTitle>
            <AlertDialogDescription>
              All branches and data in <strong>{project.name}</strong> will be
              permanently removed. This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              disabled={deleting}
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
