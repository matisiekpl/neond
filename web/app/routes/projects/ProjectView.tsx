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
import { BookDown, Cloud, Copy, GitBranchPlus, Loader2, MoreVertical, Pencil, Play, Square, Trash2 } from "lucide-react"
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/hover-card"
import type { BranchStatus } from "~/types/models/branch"

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B"
  const units = ["B", "KB", "MB", "GB", "TB"]
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${(bytes / 1024 ** i).toFixed(i === 0 ? 0 : 1)} ${units[i]}`
}

const STATUS_CONFIG: Record<BranchStatus, { label: string; className: string }> = {
  running:  { label: "Running",  className: "bg-green-500" },
  starting: { label: "Starting", className: "bg-amber-400" },
  stopping: { label: "Stopping", className: "bg-amber-400" },
  stopped:  { label: "Stopped",  className: "bg-muted-foreground" },
  failed:   { label: "Failed",   className: "bg-destructive" },
}
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
  const { branches, loading: branchesLoading, fetchBranches, createBranch, renameBranch, startEndpoint, stopEndpoint, deleteBranch } = useBranchStore(
    useShallow((s) => ({
      branches: s.branches,
      loading: s.loading,
      fetchBranches: s.fetchBranches,
      createBranch: s.createBranch,
      renameBranch: s.renameBranch,
      startEndpoint: s.startEndpoint,
      stopEndpoint: s.stopEndpoint,
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

  const [renameOpen, setRenameOpen] = React.useState(false)
  const [renameBranch_, setRenameBranch_] = React.useState<Branch | null>(null)
  const [renameName, setRenameName] = React.useState("")
  const [renaming, setRenaming] = React.useState(false)

  function openRename(branch: Branch) {
    setRenameBranch_(branch)
    setRenameName(branch.name)
    setRenameOpen(true)
  }

  async function submitRename() {
    if (!selectedOrganizationId || !projectId || !renameBranch_) return
    const trimmed = renameName.trim()
    if (!trimmed || trimmed === renameBranch_.name) return
    setRenaming(true)
    try {
      await renameBranch(selectedOrganizationId, projectId, renameBranch_.id, trimmed)
      setRenameOpen(false)
    } catch (e) {
      toast.error(getAppError(e))
    } finally {
      setRenaming(false)
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
                  <TableHead className="w-28">Status</TableHead>
                  <TableHead className="w-24">Size</TableHead>
                  <TableHead className="w-44">Durability</TableHead>
                  <TableHead className="w-28">Created</TableHead>
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
                    <TableCell className="w-28">
                      {(() => {
                        const s = STATUS_CONFIG[branch.endpoint_status]
                        return (
                          <span className="flex items-center gap-1.5">
                            <span className={`inline-block size-2 shrink-0 rounded-full ${s.className}`} />
                            <span className="text-xs text-muted-foreground">{s.label}</span>
                          </span>
                        )
                      })()}
                    </TableCell>
                    <TableCell className="w-24 text-xs text-muted-foreground">
                      {formatBytes(branch.current_logical_size)}
                    </TableCell>
                    <TableCell className="w-44">
                      {(() => {
                        const synced = branch.remote_consistent_lsn_visible === branch.last_record_lsn
                        return (
                          <HoverCard openDelay={0} closeDelay={500}>
                            <HoverCardTrigger asChild>
                              <span className={`inline-flex cursor-default items-center rounded border px-1.5 py-0.5 text-xs font-medium ${synced ? "border-green-500/30 bg-green-500/10 text-green-600" : "border-amber-500/30 bg-amber-500/10 text-amber-600"}`}>
                                {synced ? "checkpointed" : "awaiting checkpoint"}
                              </span>
                            </HoverCardTrigger>
                            <HoverCardContent className="w-72">
                              <div className="flex flex-col gap-4">
                                <div className="flex w-full items-center gap-3">
                                  <div className="flex w-20 flex-col items-center gap-1">
                                    <span className="text-[10px] font-medium text-muted-foreground">WAL</span>
                                    <BookDown className="size-5 shrink-0 text-green-500" />
                                    <span className="w-full truncate text-center font-mono text-[10px] text-muted-foreground">{branch.last_record_lsn}</span>
                                  </div>
                                  <div className="flex-1 self-start pt-[22px]">
                                    <div
                                      className="h-px w-full"
                                      style={{
                                        backgroundImage: `radial-gradient(circle, ${synced ? "rgb(34 197 94)" : "rgb(156 163 175)"} 1px, transparent 1px)`,
                                        backgroundSize: "6px 1px",
                                        animation: "dash 1s linear infinite",
                                      }}
                                    />
                                  </div>
                                  <div className="flex w-20 flex-col items-center gap-1">
                                    <span className="text-[10px] font-medium text-muted-foreground">Storage</span>
                                    <Cloud className={`size-5 shrink-0 ${synced ? "text-green-500" : "text-muted-foreground"}`} />
                                    <span className="w-full truncate text-center font-mono text-[10px] text-muted-foreground">{branch.remote_consistent_lsn_visible}</span>
                                  </div>
                                </div>
                                <style>{`@keyframes dash { from { background-position: 0 0; } to { background-position: 12px 0; } }`}</style>
                                <p className="text-center text-xs text-muted-foreground">
                                  {synced
                                    ? "All WAL records have been durably stored."
                                    : <>WAL is ahead of durable storage.<br />Pageserver is awaiting checkpoint.</>}
                                </p>
                              </div>
                            </HoverCardContent>
                          </HoverCard>
                        )
                      })()}
                    </TableCell>
                    <TableCell className="w-28 text-muted-foreground">
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
                        <DropdownMenuContent align="end" className="w-44">
                          {(branch.endpoint_status === "stopped" || branch.endpoint_status === "failed") && (
                            <DropdownMenuItem onClick={() => void startEndpoint(selectedOrganizationId!, projectId!, branch.id)}>
                              <Play className="size-4" />
                              Start
                            </DropdownMenuItem>
                          )}
                          {branch.endpoint_status === "running" && (
                            <DropdownMenuItem onClick={() => void stopEndpoint(selectedOrganizationId!, projectId!, branch.id)}>
                              <Square className="size-4" />
                              Stop
                            </DropdownMenuItem>
                          )}
                          {(branch.endpoint_status === "starting" || branch.endpoint_status === "stopping") && (
                            <DropdownMenuItem disabled>
                              <Loader2 className="size-4 animate-spin" />
                              {branch.endpoint_status === "starting" ? "Starting…" : "Stopping…"}
                            </DropdownMenuItem>
                          )}
                          <DropdownMenuItem
                            disabled={!branch.connection_string}
                            onClick={() => {
                              if (branch.connection_string) {
                                void navigator.clipboard.writeText(branch.connection_string)
                                toast.success("Connection string copied")
                              }
                            }}
                          >
                            <Copy className="size-4" />
                            Copy connection string
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => openRename(branch)}>
                            <Pencil className="size-4" />
                            Rename
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => openBranchFrom(branch)}>
                            <GitBranchPlus className="size-4" />
                            Branch from here
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            className="text-destructive focus:text-destructive"
                            onClick={() => openDelete(branch.id)}
                          >
                            <Trash2 className="size-4" />
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

      <Dialog open={renameOpen} onOpenChange={setRenameOpen}>
        <DialogContent className="sm:max-w-md" onOpenAutoFocus={(e) => e.preventDefault()}>
          <DialogHeader>
            <DialogTitle>Rename branch</DialogTitle>
          </DialogHeader>
          <div className="grid gap-2 py-2">
            <Label htmlFor="rename-branch-name">Name</Label>
            <Input
              id="rename-branch-name"
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
            <Button variant="outline" type="button" onClick={() => setRenameOpen(false)}>
              Cancel
            </Button>
            <Button
              type="button"
              disabled={renaming || !renameName.trim() || renameName.trim() === renameBranch_?.name}
              onClick={() => void submitRename()}
            >
              {renaming && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

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
