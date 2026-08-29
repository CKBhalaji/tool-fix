"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { continueWithGoogle } from "@/services/auth";
import { PrimaryButton, SecondaryButton } from "@/components/ui";
import type { UserRole } from "@/types";

/**
 * Post-login onboarding: pick CUSTOMER or MECHANIC. The backend decides
 * and stores the role; the browser request is only a preference.
 */
export default function RegisterPage() {
  const { me, loading, refresh } = useAuth();
  const router = useRouter();
  const [role, setRole] = useState<UserRole>("customer");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!loading && !me) {
      router.replace("/login");
    }
    if (me?.user.onboarding_status === "complete") {
      router.replace(
        me.user.role === "mechanic" ? "/mechanic/dashboard" : "/customer/dashboard",
      );
    }
  }, [me, loading, router]);

  async function submit() {
    setSubmitting(true);
    setError(null);
    try {
      const { completeOnboarding } = await import("@/services/auth");
      await completeOnboarding(role);
      await refresh();
      router.replace(role === "mechanic" ? "/mechanic/dashboard" : "/customer/dashboard");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Onboarding failed");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <main className="flex flex-1 items-center justify-center px-4">
      <div className="w-full max-w-md rounded-2xl border border-slate-200 bg-white p-8 shadow-sm">
        <h1 className="text-xl font-bold text-slate-900">Set up your account</h1>
        <p className="mt-1 text-sm text-slate-600">How will you use ToolFix?</p>

        {!me ? (
          <div className="mt-6">
            <p className="text-sm text-slate-500">Please sign in first.</p>
            <div className="mt-4">
              <SecondaryButton onClick={continueWithGoogle}>Continue with Google</SecondaryButton>
            </div>
          </div>
        ) : (
          <>
            <div className="mt-6 grid gap-3">
              <RoleOption
                checked={role === "customer"}
                onChange={() => setRole("customer")}
                title="I need help"
                description="Request roadside assistance when your vehicle breaks down."
              />
              <RoleOption
                checked={role === "mechanic"}
                onChange={() => setRole("mechanic")}
                title="I fix vehicles"
                description="Receive nearby breakdown requests and bid for jobs."
              />
            </div>
            {error ? <p className="mt-3 text-sm text-red-600">{error}</p> : null}
            <div className="mt-6">
              <PrimaryButton onClick={submit} disabled={submitting} className="w-full">
                {submitting ? "Saving…" : "Continue"}
              </PrimaryButton>
            </div>
          </>
        )}
      </div>
    </main>
  );
}

function RoleOption({
  checked,
  onChange,
  title,
  description,
}: {
  checked: boolean;
  onChange: () => void;
  title: string;
  description: string;
}) {
  return (
    <label
      className={`flex cursor-pointer items-start gap-3 rounded-xl border p-4 transition ${
        checked ? "border-blue-500 bg-blue-50" : "border-slate-200 hover:bg-slate-50"
      }`}
    >
      <input type="radio" checked={checked} onChange={onChange} className="mt-1" />
      <span>
        <span className="block font-semibold text-slate-900">{title}</span>
        <span className="block text-xs text-slate-500">{description}</span>
      </span>
    </label>
  );
}
