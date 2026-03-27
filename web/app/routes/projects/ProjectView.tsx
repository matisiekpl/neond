import * as React from "react"
import { useParams, useNavigate } from "react-router"
import { useShallow } from "zustand/react/shallow"
import { useProjectStore } from "~/stores/project-store"
import { useOrganizationStore } from "~/stores/organization-store"
import { useBranchStore } from "~/stores/branch-store"
import { getAppError } from "~/lib/errors"
import { toast } from "sonner"
import { Spinner } from "~/components/ui/spinner"
import { Button } from "~/components/ui/button"
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "~/components/ui/table"
import { Loader2, MoreVertical } from "lucide-react"
import type { Branch } from "~/types/models/branch"

type BranchNode = Branch & { children: BranchNode[] }

function buildTree(branches: Branch[]): BranchNode[] {
  const map = new Map<string, BranchNode>()
  for (const b of branches) map.set(b.id, { ...b, children: [] })
  const roots: BranchNode[] = []
  for (const node of map.values()) {
    if (node.parent_branch_id && map.has(node.parent_branch_id)) {
      map.get(node.parent_branch_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  }
  return roots
}

function flattenTree(nodes: BranchNode[], depth = 0): { branch: BranchNode; depth: number }[] {
  const result: { branch: BranchNode; depth: number }[] = []
  for (const node of nodes) {
    result.push({ branch: node, depth })
    result.push(...flattenTree(node.children, depth + 1))
  }
  return result
}

export default function ProjectViewRoute() {
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()

  const selectedOrganizationId = useOrganizationStore((s) => s.selectedOrganizationId)
  const { projects, loading: projectsLoading, fetchProjects } = useProjectStore(
    useShallow((s) => ({
      projects: s.projects,
      loading: s.loading,
      fetchProjects: s.fetchProjects,
    })),
  )
  const { branches, loading: branchesLoading, fetchBranches, createBranch, deleteBranch } = useBranchStore(
    useShallow((s) => ({
      branches: s.branches,
      loading: s.loading,
      fetchBranches: s.fetchBranches,
      createBranch: s.createBranch,
      deleteBranch: s.deleteBranch,
    })),
  )

  const [createOpen, setCreateOpen] = React.useState(false)
  const [newBranchName, setNewBranchName] = React.useState("")
  const [creating, setCreating] = React.useState(false)

  const [deleteOpen, setDeleteOpen] = React.useState(false)
  const [deleteId, setDeleteId] = React.useState<string | null>(null)
  const [deleting, setDeleting] = React.useState(false)

  function openDelete(branchId: string) {
    setDeleteId(branchId)
    setDeleteOpen(true)
  }

  async function confirmDelete() {
    if (!selectedOrganizationId || !projectId || !deleteId) return
    setDeleting(true)
    try {
      await deleteBranch(selectedOrganizationId, projectId, deleteId)
      setDeleteOpen(false)
    } finally {
      setDeleting(false)
    }
  }

  function openCreate() {
    setNewBranchName(branches.length === 0 ? "main" : "")
    setCreateOpen(true)
  }

  const [branchFromOpen, setBranchFromOpen] = React.useState(false)
  const [branchFromParent, setBranchFromParent] = React.useState<Branch | null>(null)
  const [branchFromName, setBranchFromName] = React.useState("")
  const [branchFromCreating, setBranchFromCreating] = React.useState(false)

  function openBranchFrom(parent: Branch) {
    setBranchFromParent(parent)
    setBranchFromName("")
    setBranchFromOpen(true)
  }

  async function submitBranchFrom() {
    if (!selectedOrganizationId || !projectId || !branchFromParent) return
    const trimmed = branchFromName.trim()
    if (!trimmed) return
    setBranchFromCreating(true)
    try {
      await createBranch(selectedOrganizationId, projectId, trimmed, branchFromParent.id)
      setBranchFromOpen(false)
    } catch (e) {
      toast.error(getAppError(e))
    } finally {
      setBranchFromCreating(false)
    }
  }

  React.useEffect(() => {
    if (selectedOrganizationId) {
      void fetchProjects(selectedOrganizationId)
    }
  }, [selectedOrganizationId, fetchProjects])

  React.useEffect(() => {
    if (!selectedOrganizationId || !projectId) return
    void fetchBranches(selectedOrganizationId, projectId)
    const interval = setInterval(() => {
      void fetchBranches(selectedOrganizationId, projectId, true)
    }, 3000)
    return () => clearInterval(interval)
  }, [selectedOrganizationId, projectId, fetchBranches])

  const project = projects.find((p) => p.id === projectId)

  React.useEffect(() => {
    if (project) {
      document.title = `${project.name} — neond`
    }
  }, [project])

  React.useEffect(() => {
    if (!createOpen) setNewBranchName("")
  }, [createOpen])

  async function submitCreateBranch() {
    if (!selectedOrganizationId || !projectId) return
    const trimmed = newBranchName.trim()
    if (!trimmed) return
    setCreating(true)
    try {
      await createBranch(selectedOrganizationId, projectId, trimmed)
      setCreateOpen(false)
    } catch (e) {
      toast.error(getAppError(e))
    } finally {
      setCreating(false)
    }
  }

  if (projectsLoading) {
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
        <p className="mt-1 text-sm text-muted-foreground">
          This project may have been deleted or you don't have access to it.
        </p>
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

  const rows = flattenTree(buildTree(branches))

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-lg font-semibold">{project.name}</h1>
        <p className="text-sm text-muted-foreground">
          PostgreSQL {project.pg_version.replace(/^V/i, "")}
        </p>
      </div>

      <div className="border bg-card p-6">
        <dl className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <dt className="text-xs font-medium text-muted-foreground">Name</dt>
            <dd className="mt-1 text-sm">{project.name}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium text-muted-foreground">PostgreSQL version</dt>
            <dd className="mt-1 text-sm">{project.pg_version.replace(/^V/i, "")}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium text-muted-foreground">Created</dt>
            <dd className="mt-1 text-sm">
              {new Date(project.created_at).toLocaleDateString(undefined, {
                year: "numeric",
                month: "long",
                day: "numeric",
              })}
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium text-muted-foreground">Project ID</dt>
            <dd className="mt-1 font-mono text-xs text-muted-foreground">{project.id}</dd>
          </div>
        </dl>
      </div>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold">Branches</h2>
          <Button type="button" size="sm" onClick={openCreate} disabled={creating}>
            {creating && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
            New branch
          </Button>
        </div>

        {branchesLoading && branches.length === 0 ? (
          <div className="flex justify-center py-8">
            <Spinner className="size-5" />
          </div>
        ) : branches.length === 0 ? (
          <div className="flex min-h-24 w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-8 text-center">
            <p className="text-sm text-muted-foreground">No branches yet.</p>
          </div>
        ) : (
          <div className={`relative border transition-opacity ${branchesLoading ? "opacity-50" : ""}`}>
            {branchesLoading && (
              <div className="absolute inset-0 z-10 flex items-center justify-center">
                <Spinner className="size-5" />
              </div>
            )}
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead className="text-right">Last record LSN</TableHead>
                  <TableHead className="text-right">Consistent LSN</TableHead>
                  <TableHead className="text-right">Created</TableHead>
                  <TableHead className="w-10" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map(({ branch, depth }) => (
                  <TableRow key={branch.id}>
                    <TableCell className="font-medium">
                      <span
                        className="flex items-center gap-1.5"
                        style={{ paddingLeft: depth * 20 }}
                      >
                        {depth > 0 && (
                          <span className="shrink-0 text-muted-foreground">╰</span>
                        )}
                        <span>{branch.name}</span>
                      </span>
                    </TableCell>
                    <TableCell className="text-right font-mono text-xs text-muted-foreground">
                      {branch.last_record_lsn}
                    </TableCell>
                    <TableCell className="text-right font-mono text-xs text-muted-foreground">
                      {branch.remote_consistent_lsn_visible}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {new Date(branch.created_at).toLocaleDateString(undefined, {
                        year: "numeric",
                        month: "short",
                        day: "numeric",
                      })}
                    </TableCell>
                    <TableCell>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon" className="size-7">
                            <MoreVertical className="size-4" />
                            <span className="sr-only">Open menu</span>
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => openBranchFrom(branch)}>
                            Branch from here
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            className="text-destructive focus:text-destructive"
                            onClick={() => openDelete(branch.id)}
                          >
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}
      </div>


      <AlertDialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete branch?</AlertDialogTitle>
            <AlertDialogDescription>
              This branch and all its data will be permanently removed. This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              disabled={deleting}
              onClick={() => void confirmDelete()}
            >
              {deleting && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Delete branch
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Dialog open={branchFromOpen} onOpenChange={setBranchFromOpen}>
        <DialogContent className="sm:max-w-md" onOpenAutoFocus={(e) => e.preventDefault()}>
          <DialogHeader>
            <DialogTitle>Branch from {branchFromParent?.name}</DialogTitle>
          </DialogHeader>
          <div className="grid gap-2 py-2">
            <Label htmlFor="branch-from-name">New branch name</Label>
            <Input
              id="branch-from-name"
              value={branchFromName}
              onChange={(e) => setBranchFromName(e.target.value)}
              placeholder="my-branch"
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault()
                  void submitBranchFrom()
                }
              }}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" type="button" onClick={() => setBranchFromOpen(false)}>
              Cancel
            </Button>
            <Button
              type="button"
              disabled={branchFromCreating || !branchFromName.trim()}
              onClick={() => void submitBranchFrom()}
            >
              {branchFromCreating && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Create branch
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent className="sm:max-w-md" onOpenAutoFocus={(e) => e.preventDefault()}>
          <DialogHeader>
            <DialogTitle>New branch</DialogTitle>
          </DialogHeader>
          <div className="grid gap-2 py-2">
            <Label htmlFor="new-branch-name">Name</Label>
            <Input
              id="new-branch-name"
              value={newBranchName}
              onChange={(e) => setNewBranchName(e.target.value)}
              placeholder="my-branch"
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault()
                  void submitCreateBranch()
                }
              }}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" type="button" onClick={() => setCreateOpen(false)}>
              Cancel
            </Button>
            <Button
              type="button"
              disabled={creating || !newBranchName.trim()}
              onClick={() => void submitCreateBranch()}
            >
              {creating && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Create
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
