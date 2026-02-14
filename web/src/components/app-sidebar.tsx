"use client"

import * as React from "react"
import Image from "next/image"
import { Calendar, CheckSquare, Inbox, List, Tag } from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar"

// Sample data
const data = {
  navMain: [
    {
      title: "Inbox",
      url: "/dashboard",
      icon: Inbox,
    },
    {
      title: "Today",
      url: "#",
      icon: Calendar,
    },
    {
      title: "Upcoming",
      url: "#",
      icon: Calendar,
    },
    {
      title: "Completed",
      url: "#",
      icon: CheckSquare,
    },
  ],
  lists: [
    {
      name: "Personal",
      url: "#",
      icon: List,
    },
    {
      name: "Work",
      url: "#",
      icon: List,
    },
    {
      name: "Shopping",
      url: "#",
      icon: Tag,
    },
  ],
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar variant="floating" collapsible="icon" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" asChild>
              <a href="#">
                <Image
                  src="/logo.jpg"
                  alt="Go2Do"
                  width={32}
                  height={32}
                  className="rounded-lg"
                />
                <div className="flex flex-col gap-0.5 leading-none">
                  <span className="font-semibold">Go2Do</span>
                </div>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Focus</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {data.navMain.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild tooltip={item.title}>
                    <a href={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>Lists</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {data.lists.map((item) => (
                <SidebarMenuItem key={item.name}>
                  <SidebarMenuButton asChild tooltip={item.name}>
                    <a href={item.url}>
                      <item.icon />
                      <span>{item.name}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  )
}
