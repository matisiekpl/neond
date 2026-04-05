import * as React from "react"
import { Link, Navigate, useSearchParams } from "react-router"
import { useForm } from "react-hook-form"
import { useShallow } from "zustand/react/shallow"
import { useAuthStore } from "~/stores/auth-store"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { Spinner } from "~/components/ui/spinner"

type RegisterFields = {
  name: string
  email: string
  password: string
}

export default function RegisterRoute() {
  const { user, initialized, register: registerUser } = useAuthStore(
    useShallow((s) => ({
      user: s.user,
      initialized: s.initialized,
      register: s.register,
    })),
  )
  const [searchParams] = useSearchParams()
  const emailLocked = Boolean(searchParams.get("email"))
  const { register, handleSubmit, setValue, formState: { isSubmitting } } = useForm<RegisterFields>({
    defaultValues: { name: "", email: "", password: "" },
  })

  React.useEffect(() => {
    document.title = "Register — neond"
  }, [])

  React.useEffect(() => {
    const q = searchParams.get("email")
    if (q) setValue("email", q)
  }, [searchParams, setValue])

  async function onSubmit({ name, email, password }: RegisterFields) {
    await registerUser(name, email, password)
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
        <Label className="text-3xl">Create an account</Label>
        <Label className="text-muted-foreground">Get started with neond</Label>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit(onSubmit)}>
          <div className="flex flex-col gap-2">
            <Label htmlFor="register-name">Name</Label>
            <Input
              id="register-name"
              type="text"
              {...register("name")}
              autoComplete="name"
              required
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="register-email">Email</Label>
            <Input
              id="register-email"
              type="email"
              {...register("email")}
              disabled={emailLocked}
              autoComplete="email"
              required
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="register-password">Password</Label>
            <Input
              id="register-password"
              type="password"
              {...register("password")}
              autoComplete="new-password"
              required
            />
          </div>
          <Button type="submit" className="w-full" disabled={isSubmitting}>
            {isSubmitting ? <Spinner className="mr-2" /> : null}
            Sign up
          </Button>
          <span className="text-center text-sm text-muted-foreground">
            Already have an account?{" "}
            <Link to="/login" className="underline hover:text-foreground">
              Sign in here
            </Link>
          </span>
        </form>
      </div>
    </div>
  )
}