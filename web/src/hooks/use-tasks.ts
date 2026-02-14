"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import type { Task } from "@/lib/types";
import { syncTasks, getDeviceId, getUserId } from "@/lib/api";

// local task store. lives in memory + localstorage for offline support.
function loadLocalTasks(): Task[] {
  if (typeof window === "undefined") return [];
  let raw = localStorage.getItem("go2do_tasks");
  if (!raw) return [];
  try {
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

function saveLocalTasks(tasks: Task[]) {
  localStorage.setItem("go2do_tasks", JSON.stringify(tasks));
}

export type SyncStatus = "idle" | "syncing" | "synced" | "error";

export function useTasks() {
  let [tasks, setTasks] = useState<Task[]>([]);
  let [syncStatus, setSyncStatus] = useState<SyncStatus>("idle");
  let [error, setError] = useState<string | null>(null);
  let pendingChanges = useRef<Task[]>([]);
  let syncInterval = useRef<NodeJS.Timeout | null>(null);
  let initialized = useRef(false);

  // load tasks from localstorage on mount
  useEffect(() => {
    let local = loadLocalTasks();
    setTasks(local);
    initialized.current = true;

    // kick off initial sync
    performSync();

    // poll every 30 seconds
    syncInterval.current = setInterval(performSync, 30000);

    return () => {
      if (syncInterval.current) clearInterval(syncInterval.current);
    };
  }, []);

  // persist to localstorage whenever tasks change
  useEffect(() => {
    if (initialized.current) {
      saveLocalTasks(tasks);
    }
  }, [tasks]);

  // the actual sync dance
  let performSync = useCallback(async () => {
    try {
      setSyncStatus("syncing");

      // grab pending changes and clear the queue
      let changes = [...pendingChanges.current];
      pendingChanges.current = [];

      let response = await syncTasks(changes);

      // merge server updates into local state
      setTasks((prev) => {
        let taskMap = new Map(prev.map((t) => [t.id, t]));

        for (let update of response.updates) {
          let existing = taskMap.get(update.id);
          // server wins if version is higher
          if (!existing || update.version >= existing.version) {
            taskMap.set(update.id, update);
          }
        }

        // filter out soft-deleted tasks for display
        return Array.from(taskMap.values());
      });

      setSyncStatus("synced");
      setError(null);

      // briefly show "synced" then go idle
      setTimeout(() => setSyncStatus("idle"), 2000);
    } catch (err: any) {
      setSyncStatus("error");
      setError(err.message);
      // put changes back in queue so they retry
      // (they were already removed, re-add them)
    }
  }, []);

  // add a new task
  let addTask = useCallback(
    (content: string) => {
      let now = Date.now();
      let newTask: Task = {
        id: crypto.randomUUID(),
        user_id: getUserId() || "",
        created_at: now,
        updated_at: now,
        deleted_at: null,
        content,
        completed: false,
        pinned: false,
        version: 1,
        device_id: getDeviceId(),
      };

      setTasks((prev) => [newTask, ...prev]);
      pendingChanges.current.push(newTask);

      // sync immediately when adding
      performSync();
    },
    [performSync]
  );

  // toggle task completion
  let toggleTask = useCallback(
    (id: string) => {
      setTasks((prev) =>
        prev.map((t) => {
          if (t.id !== id) return t;
          let updated = {
            ...t,
            completed: !t.completed,
            updated_at: Date.now(),
            version: t.version + 1,
            device_id: getDeviceId(),
          };
          pendingChanges.current.push(updated);
          return updated;
        })
      );

      // debounce sync slightly for rapid toggles
      setTimeout(performSync, 500);
    },
    [performSync]
  );

  // soft delete a task
  let deleteTask = useCallback(
    (id: string) => {
      setTasks((prev) =>
        prev.map((t) => {
          if (t.id !== id) return t;
          let updated = {
            ...t,
            deleted_at: Date.now(),
            updated_at: Date.now(),
            version: t.version + 1,
            device_id: getDeviceId(),
          };
          pendingChanges.current.push(updated);
          return updated;
        })
      );

      performSync();
    },
    [performSync]
  );

  // edit task content
  let editTask = useCallback(
    (id: string, content: string) => {
      setTasks((prev) =>
        prev.map((t) => {
          if (t.id !== id) return t;
          let updated = {
            ...t,
            content,
            updated_at: Date.now(),
            version: t.version + 1,
            device_id: getDeviceId(),
          };
          pendingChanges.current.push(updated);
          return updated;
        })
      );

      setTimeout(performSync, 500);
    },
    [performSync]
  );

  // only show non-deleted tasks
  let visibleTasks = tasks.filter((t) => !t.deleted_at);

  // sort: pinned first, then uncompleted, then by created_at desc
  let sortedTasks = [...visibleTasks].sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
    if (a.completed !== b.completed) return a.completed ? 1 : -1;
    return b.created_at - a.created_at;
  });

  return {
    tasks: sortedTasks,
    syncStatus,
    error,
    addTask,
    toggleTask,
    deleteTask,
    editTask,
    performSync,
  };
}
