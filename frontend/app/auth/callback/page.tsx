"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";

/**
 * Post-login landing target (the backend redirects here after setting the
 * HttpOnly cookies). Routes by role and onboarding state.
 */
export default function AuthCallbackPage() {
  const { me, loading, refresh } = useAuth();
  const router = useRouter();

  useEffect(() => {
    void refresh();
  }, [refresh]);

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
    <>
      <Header />
      <main className="flex flex-1 items-center justify-center">
        <p className="text-sm text-slate-500">Signing you in…</p>
      </main>
    </>
  );
}
