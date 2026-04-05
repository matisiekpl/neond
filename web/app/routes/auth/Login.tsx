import * as React from "react"
import { Link, Navigate, useSearchParams } from "react-router"
import { useForm } from "react-hook-form"
import { useShallow } from "zustand/react/shallow"
import { useAuthStore } from "~/stores/auth-store"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { Spinner } from "~/components/ui/spinner"

type LoginFields = {
  email: string
  password: string
}

export default function LoginRoute() {
  const { user, initialized, login } = useAuthStore(
    useShallow((s) => ({
      user: s.user,
      initialized: s.initialized,
      login: s.login,
    })),
  )
  const [searchParams] = useSearchParams()
  const emailLocked = Boolean(searchParams.get("email"))
  const { register, handleSubmit, setValue, formState: { isSubmitting } } = useForm<LoginFields>({
    defaultValues: { email: "", password: "" },
  })

  React.useEffect(() => {
    document.title = "Login — neond"
  }, [])

  React.useEffect(() => {
    const q = searchParams.get("email")
    if (q) setValue("email", q)
  }, [searchParams, setValue])

  async function onSubmit({ email, password }: LoginFields) {
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
        <form className="flex flex-col gap-4" onSubmit={handleSubmit(onSubmit)}>
          <div className="flex flex-col gap-2">
            <Label htmlFor="login-email">Email</Label>
            <Input
              id="login-email"
              type="email"
              {...register("email")}
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
              {...register("password")}
              autoComplete="current-password"
              required
            />
          </div>
          <Button type="submit" className="w-full" disabled={isSubmitting}>
            {isSubmitting ? <Spinner className="mr-2" /> : null}
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