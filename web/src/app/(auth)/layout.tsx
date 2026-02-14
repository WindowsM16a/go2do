"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { isAuthenticated } from "@/lib/api";

export default function AuthLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  let router = useRouter();

  useEffect(function() {
    if (isAuthenticated()) {
      router.push("/dashboard");
    }
  }, [router]);

  return (
    <div className="flex min-h-screen items-center justify-center bg-bg-primary p-4 text-text-primary">
      <div className="w-full max-w-md space-y-8 rounded-xl border border-border-subtle bg-bg-secondary p-8 shadow-2xl">
        <div className="text-center">
          <h2 className="mt-2 text-3xl font-bold tracking-tight text-text-primary">
            Go2Do
          </h2>
        </div>
        {children}
      </div>
    </div>
  );
}
