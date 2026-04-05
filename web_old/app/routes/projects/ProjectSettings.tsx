import * as React from "react"
import {useParams, useNavigate} from "react-router"
import {useForm, Controller} from "react-hook-form"
import {useShallow} from "zustand/react/shallow"
import {useProjectStore} from "~/stores/project-store"
import {useOrganizationStore} from "~/stores/organization-store"
import {projectsApi} from "~/api/projects"
import {getAppError} from "~/lib/errors"
import {toast} from "sonner"
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
import {Loader2} from "lucide-react"
import {Button} from "~/components/ui/button"
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "~/components/ui/card"
import {Input} from "~/components/ui/input"
import {MegabytesInput} from "~/components/ui/megabytes-input"
import {Label} from "~/components/ui/label"
import {Spinner} from "~/components/ui/spinner"
import {
    SliderRoot,
    SliderTrack,
    SliderRange,
    SliderThumb,
} from "~/components/ui/slider"

const PITR_PRESETS = [
    {label: "0h", value: ""},
    {label: "1h", value: "1h"},
    {label: "6h", value: "6h"},
    {label: "12h", value: "12h"},
    {label: "1 day", value: "1day"},
    {label: "3 days", value: "3days"},
    {label: "7 days", value: "7days"},
    {label: "14 days", value: "14days"},
    {label: "30 days", value: "30days"},
]

const GC_PERIOD_PRESETS = [
    {label: "10m", value: "10m"},
    {label: "30m", value: "30m"},
    {label: "1h", value: "1h"},
    {label: "2h", value: "2h"},
    {label: "6h", value: "6h"},
    {label: "12h", value: "12h"},
    {label: "24h", value: "1day"},
]

const CHECKPOINT_TIMEOUT_PRESETS = [
    {label: "1m", value: "1m"},
    {label: "5m", value: "5m"},
    {label: "10m", value: "10m"},
    {label: "30m", value: "30m"},
    {label: "1h", value: "1h"},
]

function presetToIndex(presets: { value: string }[], v: string) {
    const idx = presets.findIndex((p) => p.value === v)
    return idx === -1 ? 0 : idx
}

function SliderField({
                         presets,
                         value,
                         onChange,
                         ariaLabel,
                     }: {
    presets: { label: string; value: string }[]
    value: string
    onChange: (v: string) => void
    ariaLabel: string
}) {
    return (
        <div className="space-y-3">
            <SliderRoot
                min={0}
                max={presets.length - 1}
                step={1}
                value={[presetToIndex(presets, value)]}
                onValueChange={([idx]) => onChange(presets[idx].value)}
            >
                <SliderTrack>
                    <SliderRange/>
                </SliderTrack>
                <SliderThumb aria-label={ariaLabel}/>
            </SliderRoot>
            <div className="relative h-4">
                {presets.map((p, i) => {
                    const pct = (i / (presets.length - 1)) * 100
                    const isFirst = i === 0
                    const isLast = i === presets.length - 1
                    return (
                        <span
                            key={p.label}
                            className="absolute text-xs text-muted-foreground"
                            style={{
                                left: isLast ? undefined : `${pct}%`,
                                right: isLast ? "0%" : undefined,
                                transform: isFirst || isLast ? undefined : "translateX(-50%)",
                            }}
                        >
              {p.label}
            </span>
                    )
                })}
            </div>
        </div>
    )
}

type NameFields = { name: string }
type GcFields = { gcPeriod: string; gcHorizon: string }
type PitrFields = { pitrInterval: string }
type CheckpointFields = { checkpointDistance: string; checkpointTimeout: string }

export default function ProjectSettingsRoute() {
    const {projectId} = useParams<{ projectId: string }>()
    const navigate = useNavigate()

    const selectedOrganizationId = useOrganizationStore((s) => s.selectedOrganizationId)
    const {projects, loading, fetchProjects, updateProject, deleteProject} =
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

    const [deleteOpen, setDeleteOpen] = React.useState(false)
    const [deleting, setDeleting] = React.useState(false)
    const [configLoading, setConfigLoading] = React.useState(false)

    const nameForm = useForm<NameFields>({ defaultValues: { name: "" } })
    const gcForm = useForm<GcFields>({ defaultValues: { gcPeriod: "", gcHorizon: "" } })
    const pitrForm = useForm<PitrFields>({ defaultValues: { pitrInterval: "7days" } })
    const checkpointForm = useForm<CheckpointFields>({ defaultValues: { checkpointDistance: "", checkpointTimeout: "" } })

    React.useEffect(() => {
        if (selectedOrganizationId) {
            void fetchProjects(selectedOrganizationId)
        }
    }, [selectedOrganizationId, fetchProjects])

    React.useEffect(() => {
        if (project) {
            nameForm.reset({ name: project.name })
            document.title = `Settings — ${project.name} — neond`
        }
    }, [project?.id])

    React.useEffect(() => {
        if (!selectedOrganizationId || !projectId) return
        setConfigLoading(true)
        projectsApi
            .get(selectedOrganizationId, projectId)
            .then((p) => {
                gcForm.reset({
                    gcPeriod: p.gc_period ?? "",
                    gcHorizon: p.gc_horizon !== undefined ? String(p.gc_horizon) : "",
                })
                pitrForm.reset({
                    pitrInterval: p.pitr_interval ?? "7days",
                })
                checkpointForm.reset({
                    checkpointDistance: p.checkpoint_distance !== undefined ? String(p.checkpoint_distance) : "",
                    checkpointTimeout: p.checkpoint_timeout ?? "",
                })
            })
            .catch(() => {})
            .finally(() => setConfigLoading(false))
    }, [selectedOrganizationId, projectId])

    async function saveName({ name }: NameFields) {
        if (!selectedOrganizationId || !projectId) return
        const trimmed = name.trim()
        if (!trimmed || trimmed === project?.name) return
        try {
            await updateProject(selectedOrganizationId, projectId, {name: trimmed})
            nameForm.reset({ name: trimmed })
        } catch (e) {
            toast.error(getAppError(e))
        }
    }

    async function saveGc({ gcPeriod, gcHorizon }: GcFields) {
        if (!selectedOrganizationId || !projectId || !project) return
        try {
            await updateProject(selectedOrganizationId, projectId, {
                gc_period: gcPeriod.trim() || undefined,
                gc_horizon: gcHorizon.trim() ? Number(gcHorizon) : undefined,
            })
            gcForm.reset({ gcPeriod, gcHorizon })
        } catch (e) {
            toast.error(getAppError(e))
        }
    }

    async function savePitr({ pitrInterval }: PitrFields) {
        if (!selectedOrganizationId || !projectId || !project) return
        try {
            await updateProject(selectedOrganizationId, projectId, {
                pitr_interval: pitrInterval.trim() || undefined,
            })
            pitrForm.reset({ pitrInterval })
        } catch (e) {
            toast.error(getAppError(e))
        }
    }

    async function saveCheckpoint({ checkpointDistance, checkpointTimeout }: CheckpointFields) {
        if (!selectedOrganizationId || !projectId || !project) return
        try {
            await updateProject(selectedOrganizationId, projectId, {
                checkpoint_distance: checkpointDistance.trim() ? Number(checkpointDistance) : undefined,
                checkpoint_timeout: checkpointTimeout.trim() || undefined,
            })
            checkpointForm.reset({ checkpointDistance, checkpointTimeout })
        } catch (e) {
            toast.error(getAppError(e))
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

    const watchedName = nameForm.watch("name")

    if (loading) {
        return (
            <div className="flex justify-center py-12">
                <Spinner className="size-6"/>
            </div>
        )
    }

    if (!project) {
        return (
            <div
                className="flex min-h-[min(360px,50vh)] w-full flex-col items-center justify-center rounded-xl border border-dashed bg-muted/30 px-6 py-12 text-center">
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
                    <form onSubmit={nameForm.handleSubmit(saveName)}>
                        <div className="grid gap-2">
                            <Label htmlFor="project-name">Name</Label>
                            <Input
                                id="project-name"
                                {...nameForm.register("name")}
                            />
                        </div>
                        <Button
                            type="submit"
                            className="mt-4"
                            disabled={nameForm.formState.isSubmitting || !watchedName.trim() || !nameForm.formState.isDirty}
                        >
                            {nameForm.formState.isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin"/>}
                            Save changes
                        </Button>
                    </form>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Garbage collection</CardTitle>
                    <CardDescription>
                        Control when old data versions are removed.
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {configLoading ? (
                        <div className="flex justify-center py-4">
                            <Spinner className="size-5"/>
                        </div>
                    ) : (
                        <form onSubmit={gcForm.handleSubmit(saveGc)}>
                            <div className="flex items-start gap-6">
                                <div className="grid gap-2 w-36">
                                    <Label htmlFor="gc-horizon">GC horizon</Label>
                                    <Controller
                                        name="gcHorizon"
                                        control={gcForm.control}
                                        render={({ field }) => (
                                            <MegabytesInput
                                                id="gc-horizon"
                                                placeholder="64"
                                                value={field.value}
                                                onChange={field.onChange}
                                            />
                                        )}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        WAL distance beyond which data can be GC'd.
                                    </p>
                                </div>
                                <div className="flex-1 grid gap-3">
                                    <Label>GC period</Label>
                                    <Controller
                                        name="gcPeriod"
                                        control={gcForm.control}
                                        render={({ field }) => (
                                            <SliderField
                                                presets={GC_PERIOD_PRESETS}
                                                value={field.value}
                                                onChange={field.onChange}
                                                ariaLabel="GC period"
                                            />
                                        )}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        How often garbage collection runs.
                                    </p>
                                </div>
                            </div>
                            <Button
                                type="submit"
                                className="mt-4"
                                disabled={gcForm.formState.isSubmitting || !gcForm.formState.isDirty}
                            >
                                {gcForm.formState.isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin"/>}
                                Save
                            </Button>
                        </form>
                    )}
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Point-in-time recovery</CardTitle>
                    <CardDescription>
                        Choose the length of your restore window. This setting enables{" "}
                        <strong>instant restore</strong> for point-in-time recovery, time
                        travel queries, and branching from past states.
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                    {configLoading ? (
                        <div className="flex justify-center py-4">
                            <Spinner className="size-5"/>
                        </div>
                    ) : (
                        <form onSubmit={pitrForm.handleSubmit(savePitr)}>
                            <Controller
                                name="pitrInterval"
                                control={pitrForm.control}
                                render={({ field }) => (
                                    <SliderField
                                        presets={PITR_PRESETS}
                                        value={field.value}
                                        onChange={field.onChange}
                                        ariaLabel="PITR interval"
                                    />
                                )}
                            />
                            <Button
                                type="submit"
                                className="mt-4"
                                disabled={pitrForm.formState.isSubmitting || !pitrForm.formState.isDirty}
                            >
                                {pitrForm.formState.isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin"/>}
                                Save
                            </Button>
                        </form>
                    )}
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Checkpointing</CardTitle>
                    <CardDescription>
                        Tune how frequently the pageserver flushes data to disk.
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {configLoading ? (
                        <div className="flex justify-center py-4">
                            <Spinner className="size-5"/>
                        </div>
                    ) : (
                        <form onSubmit={checkpointForm.handleSubmit(saveCheckpoint)}>
                            <div className="flex items-start gap-6">
                                <div className="grid gap-2 w-36">
                                    <Label htmlFor="checkpoint-distance">Checkpoint distance</Label>
                                    <Controller
                                        name="checkpointDistance"
                                        control={checkpointForm.control}
                                        render={({ field }) => (
                                            <MegabytesInput
                                                id="checkpoint-distance"
                                                placeholder="256"
                                                value={field.value}
                                                onChange={field.onChange}
                                            />
                                        )}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        Amount of WAL data between checkpoints.
                                    </p>
                                </div>
                                <div className="flex-1 grid gap-3">
                                    <Label>Checkpoint timeout</Label>
                                    <Controller
                                        name="checkpointTimeout"
                                        control={checkpointForm.control}
                                        render={({ field }) => (
                                            <SliderField
                                                presets={CHECKPOINT_TIMEOUT_PRESETS}
                                                value={field.value}
                                                onChange={field.onChange}
                                                ariaLabel="Checkpoint timeout"
                                            />
                                        )}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        Maximum time between forced checkpoints.
                                    </p>
                                </div>
                            </div>
                            <Button
                                type="submit"
                                className="mt-4"
                                disabled={checkpointForm.formState.isSubmitting || !checkpointForm.formState.isDirty}
                            >
                                {checkpointForm.formState.isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin"/>}
                                Save
                            </Button>
                        </form>
                    )}
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
                            {deleting && <Loader2 className="mr-1.5 size-3.5 animate-spin"/>}
                            Delete project
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </div>
    )
}