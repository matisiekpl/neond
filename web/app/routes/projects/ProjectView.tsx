import * as React from "react"
import { useParams, useNavigate } from "react-router"
import { useShallow } from "zustand/react/shallow"
import { useProjectStore } from "~/stores/project-store"
import { useOrganizationStore } from "~/stores/organization-store"
import { Spinner } from "~/components/ui/spinner"

export default function ProjectViewRoute() {
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()

  const selectedOrganizationId = useOrganizationStore((s) => s.selectedOrganizationId)
  const { projects, loading, fetchProjects } = useProjectStore(
    useShallow((s) => ({
      projects: s.projects,
      loading: s.loading,
      fetchProjects: s.fetchProjects,
    })),
  )

  React.useEffect(() => {
    if (selectedOrganizationId) {
      void fetchProjects(selectedOrganizationId)
    }
  }, [selectedOrganizationId, fetchProjects])

  const project = projects.find((p) => p.id === projectId)

  React.useEffect(() => {
    if (project) {
      document.title = `${project.name} — neond`
    }
  }, [project])

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
    </div>
  )
}
