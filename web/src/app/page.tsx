"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { isAuthenticated } from "@/lib/api";

export default function Home() {
  let router = useRouter();

  useEffect(function() {
    if (isAuthenticated()) {
      router.push("/dashboard");
    } else {
      router.push("/login");
    }
  }, [router]);

  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-bg-primary text-text-primary">
      <div className="animate-pulse text-text-completed">Loading Go2Do...</div>
    </main>
  );
}