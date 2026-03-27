import * as React from "react"
import { Link, Navigate, useSearchParams } from "react-router"
import { useShallow } from "zustand/react/shallow"
import { useAuthStore } from "~/stores/auth-store"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { Spinner } from "~/components/ui/spinner"

export default function LoginRoute() {
  const { user, initialized, loading, login } = useAuthStore(
    useShallow((s) => ({
      user: s.user,
      initialized: s.initialized,
      loading: s.loading,
      login: s.login,
    })),
  )
  const [searchParams] = useSearchParams()
  const emailLocked = Boolean(searchParams.get("email"))
  const [email, setEmail] = React.useState("")
  const [password, setPassword] = React.useState("")

  React.useEffect(() => {
    document.title = "Login — neond"
  }, [])

  React.useEffect(() => {
    const q = searchParams.get("email")
    if (q) {
      setEmail(q)
    }
  }, [searchParams])

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault()
    await login(email, password)
  }

  if (!initialized) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Spinner className="size-8" />
      </div>
    )
  }

  if (user) {
    return <Navigate to="/dashboard" replace />
  }

  return (
    <div className="flex h-screen">
      <div className="m-auto flex w-[330px] flex-col gap-4">
        <Label className="text-3xl">Login to neond</Label>
        <Label className="text-muted-foreground">Welcome back</Label>
        <form className="flex flex-col gap-4" onSubmit={(e) => void onSubmit(e)}>
          <div className="flex flex-col gap-2">
            <Label htmlFor="login-email">Email</Label>
            <Input
              id="login-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={emailLocked}
              autoComplete="email"
              required
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="login-password">Password</Label>
            <Input
              id="login-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="current-password"
              required
            />
          </div>
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? <Spinner className="mr-2" /> : null}
            Sign in
          </Button>
          <span className="text-center text-sm text-muted-foreground">
            Don&apos;t have an account yet?{" "}
            <Link
              to="/register"
              className="underline hover:text-foreground"
            >
              Sign up here
            </Link>
          </span>
        </form>
      </div>
    </div>
  )
}
