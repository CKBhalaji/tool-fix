"use client";

import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import { Header } from "@/components/layout/Header";
import { EarningsPayments } from "@/components/mechanic/EarningsPayments";
import { getMechanicProfile, listMyEarnings } from "@/services/mechanics";

export default function MechanicPaymentsPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const payments = useAsync(
    () => (me?.user.role === "mechanic" ? listMyEarnings() : Promise.resolve([])),
    [me?.user.id],
  );
  const profile = useAsync(
    () => (me?.user.role === "mechanic" ? getMechanicProfile() : Promise.resolve(null)),
    [me?.user.id],
  );

  if (!loading && (!me || me.user.role !== "mechanic")) {
    router.replace("/");
  }

  return (
    <>
      <Header title="My payments" />
      <main className="mx-auto w-full max-w-4xl flex-1 px-4 py-6">
        {payments.error ? (
          <p className="mb-3 text-sm text-red-600">{payments.error}</p>
        ) : null}
        {profile.data && !profile.data.is_verified ? (
          <p className="mb-3 rounded-xl bg-amber-50 p-3 text-xs text-amber-700">
            Your account is not verified yet — payments appear after jobs are completed.
          </p>
        ) : null}
        <EarningsPayments payments={payments.data ?? []} />
      </main>
    </>
  );
}
