import * as React from "react"
import { AppMainHeader } from "~/components/AppMainHeader"
import { AppSidebar } from "~/components/AppSidebar"
import { SidebarInset, SidebarProvider } from "~/components/ui/sidebar"

export function AuthenticatedLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <AppMainHeader />
        <div className="flex min-h-0 flex-1 flex-col overflow-auto p-4 md:p-6">
          {children}
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
