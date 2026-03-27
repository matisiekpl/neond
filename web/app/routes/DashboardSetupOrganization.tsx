import * as React from "react"
import { toast } from "sonner"
import { useOrganizationStore } from "~/stores/organization-store"
import { getAppError } from "~/lib/errors"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"

export default function DashboardSetupOrganizationRoute() {
  const createOrganization = useOrganizationStore((s) => s.createOrganization)
  const [name, setName] = React.useState("")
  const [creating, setCreating] = React.useState(false)

  React.useEffect(() => {
    document.title = "Create organization — neond"
  }, [])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = name.trim()
    if (!trimmed) {
      return
    }
    setCreating(true)
    try {
      await createOrganization(trimmed)
    } catch (err) {
      toast.error(getAppError(err))
    } finally {
      setCreating(false)
    }
  }

  return (
    <main className="flex min-h-svh w-full flex-col items-center justify-center bg-background px-4">
      <form
        onSubmit={(e) => void submit(e)}
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
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Acme Inc."
            autoComplete="organization"
            autoFocus
          />
        </div>
        <Button
          type="submit"
          className="w-full"
          disabled={creating || !name.trim()}
        >
          Continue
        </Button>
      </form>
    </main>
  )
}
