"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";

export default function LandingPage() {
  const { me, loading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (loading) return;
    if (!me) {
      router.replace("/login");
      return;
    }
    if (me.user.onboarding_status === "pending") {
      router.replace("/register");
      return;
    }
    router.replace(
      me.user.role === "mechanic" ? "/mechanic/dashboard" : "/customer/dashboard",
    );
  }, [me, loading, router]);

  return (
    <main className="flex flex-1 items-center justify-center">
      <p className="text-sm text-slate-500">Loading ToolFix…</p>
    </main>
  );
}
