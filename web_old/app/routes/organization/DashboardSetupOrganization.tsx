import * as React from "react"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import { useOrganizationStore } from "~/stores/organization-store"
import { getAppError } from "~/lib/errors"
import { Loader2 } from "lucide-react"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"

type FormFields = {
  name: string
}

export default function DashboardSetupOrganizationRoute() {
  const createOrganization = useOrganizationStore((s) => s.createOrganization)
  const { register, handleSubmit, watch, formState: { isSubmitting } } = useForm<FormFields>({
    defaultValues: { name: "" },
  })

  React.useEffect(() => {
    document.title = "Create organization — neond"
  }, [])

  async function onSubmit({ name }: FormFields) {
    const trimmed = name.trim()
    if (!trimmed) return
    try {
      await createOrganization(trimmed)
    } catch (err) {
      toast.error(getAppError(err))
    }
  }

  const name = watch("name")

  return (
    <main className="flex min-h-svh w-full flex-col items-center justify-center bg-background px-4">
      <form
        onSubmit={handleSubmit(onSubmit)}
        className="w-full max-w-sm space-y-6"
      >
        <div className="space-y-2 text-center">
          <h1 className="text-xl font-semibold tracking-tight">
            Name your organization
          </h1>
          <p className="text-sm text-muted-foreground">
            You need an organization before you can continue.
          </p>
        </div>
        <div className="space-y-2">
          <Label htmlFor="setup-org-name">Organization name</Label>
          <Input
            id="setup-org-name"
            {...register("name")}
            placeholder="Acme Inc."
            autoComplete="organization"
            autoFocus
          />
        </div>
        <Button
          type="submit"
          className="w-full"
          disabled={isSubmitting || !name.trim()}
        >
          {isSubmitting && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
          Continue
        </Button>
      </form>
    </main>
  )
}