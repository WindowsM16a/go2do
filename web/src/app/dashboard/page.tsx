"use client";

import { useState, useRef, useEffect } from "react";
import { useTasks, SyncStatus } from "@/hooks/use-tasks";
import { isAuthenticated, logout } from "@/lib/api";
import { useRouter } from "next/navigation";
import { Plus, Trash2, LogOut } from "lucide-react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

// sync dot color mapping, same as the desktop app
function syncDotColor(status: SyncStatus): string {
  switch (status) {
    case "syncing": return "bg-yellow";
    case "synced": return "bg-green";
    case "error": return "bg-red";
    default: return "bg-zinc-500";
  }
}

export default function DashboardPage() {
  let router = useRouter();
  let { tasks, syncStatus, addTask, toggleTask, deleteTask, performSync } = useTasks();
  let [newTask, setNewTask] = useState("");
  let inputRef = useRef<HTMLInputElement>(null);

  // redirect to login if not authenticated
  useEffect(() => {
    if (!isAuthenticated()) {
      router.push("/login");
    }
  }, [router]);

  function handleAddTask(e: React.FormEvent) {
    e.preventDefault();
    let content = newTask.trim();
    if (!content) return;
    addTask(content);
    setNewTask("");
    inputRef.current?.focus();
  }

  return (
    <div className="flex h-full flex-col">
      {/* task list */}
      <div className="flex-1 overflow-y-auto pr-2 space-y-0.5">
        {tasks.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
            <p className="text-sm">no tasks yet. add one below!</p>
          </div>
        )}

        {tasks.map((task) => (
          <div
            key={task.id}
            className={cn(
              "group flex items-center gap-3 rounded-lg border border-transparent px-3 py-2.5 text-sm transition-colors hover:bg-accent",
              task.completed && "opacity-50"
            )}
          >
            {/* checkbox */}
            <button
              onClick={() => toggleTask(task.id)}
              className={cn(
                "flex h-4 w-4 shrink-0 cursor-pointer items-center justify-center rounded-sm border transition-colors",
                task.completed
                  ? "border-green bg-green/20 text-green"
                  : "border-muted-foreground/30 hover:border-foreground"
              )}
            >
              {task.completed && (
                <svg className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                </svg>
              )}
            </button>

            {/* content */}
            <span
              className={cn(
                "flex-1",
                task.completed && "line-through text-muted-foreground"
              )}
            >
              {task.content}
            </span>

            {/* delete button, only shows on hover */}
            <button
              onClick={() => deleteTask(task.id)}
              className="cursor-pointer opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-red transition-opacity"
              title="Delete task"
            >
              <Trash2 className="h-3.5 w-3.5" />
            </button>
          </div>
        ))}
      </div>

      {/* quick input bar */}
      <div className="mt-4 shrink-0 pb-4">
        <form onSubmit={handleAddTask} className="relative">
          <div className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">
            <Plus className="h-4 w-4" />
          </div>
          <Input
            ref={inputRef}
            value={newTask}
            onChange={(e) => setNewTask(e.target.value)}
            className="pl-9 bg-background border-border focus-visible:ring-1 focus-visible:ring-ring"
            placeholder="Add a task..."
          />
        </form>
      </div>
    </div>
  );
}
