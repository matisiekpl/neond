import * as React from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"

export default function DashboardHomeRoute() {
  React.useEffect(() => {
    document.title = "Dashboard — neond"
  }, [])

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Dashboard</CardTitle>
          <CardDescription>Welcome to your workspace.</CardDescription>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Use the sidebar to switch organizations or open organization settings.
        </CardContent>
      </Card>
    </div>
  )
}
