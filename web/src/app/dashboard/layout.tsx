import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"
import { Separator } from "@/components/ui/separator"
import { CheckSquare } from "lucide-react"

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="w-full h-screen flex flex-col bg-sidebar-accent/10">
        <header className="flex h-14 items-center gap-2 border-b bg-background px-4 shrink-0">
            <SidebarTrigger className="-ml-2" />
            <Separator orientation="vertical" className="mr-2 h-4" />
            <div className="flex items-center gap-2">
                 <h1 className="text-sm font-semibold">Inbox</h1>
            </div>
            
            <div className="ml-auto flex items-center gap-2">
                {/* Sync Indicator Placeholder */}
                <div className="h-2 w-2 rounded-full bg-zinc-500" title="Sync Status: Idle"></div>
            </div>
        </header>
        <div className="flex-1 overflow-hidden p-4">
            {children}
        </div>
      </main>
    </SidebarProvider>
  )
}
