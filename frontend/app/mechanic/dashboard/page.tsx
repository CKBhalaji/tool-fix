"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import { Header } from "@/components/layout/Header";
import { AvailabilityPanel } from "@/components/mechanic/AvailabilityPanel";
import { ServiceProfileCard } from "@/components/mechanic/ServiceProfileCard";
import { Spinner } from "@/components/ui";
import { getMechanicProfile, setAvailability, updateMechanicProfile } from "@/services/mechanics";

export default function MechanicDashboardPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const profile = useAsync(() => getMechanicProfile(), [me?.user.id]);

  if (!loading && (!me || me.user.role !== "mechanic")) {
    router.replace("/");
  }

  async function toggleAvailability() {
    if (!profile.data) return;
    const next = profile.data.availability_status === "offline" ? "online" : "offline";
    await setAvailability(next);
    await profile.reload();
  }

  async function editArea(areaKm: number) {
    await updateMechanicProfile({ service_area_km: areaKm });
    await profile.reload();
  }

  return (
    <>
      <Header title="Mechanic" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        {profile.error ? (
          <p className="mb-4 text-sm text-red-600">{profile.error}</p>
        ) : null}

        {profile.loading && !profile.data ? (
          <Spinner />
        ) : (
          <div className="grid gap-5">
            <AvailabilityPanel
              profile={profile.data}
              busy={profile.loading}
              onToggle={() => void toggleAvailability()}
            />

            {profile.data && !profile.data.repair_categories.length ? (
              <p className="text-xs text-amber-600">
                Set your services and vehicle types in the profile form to appear in matching.
              </p>
            ) : null}

            <div className="grid gap-3 sm:grid-cols-3">
              <Link href="/mechanic/requests" className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:border-blue-300">
                <p className="font-semibold">Nearby requests</p>
                <p className="mt-1 text-xs text-slate-500">See breakdowns around you and submit offers.</p>
              </Link>
              <Link href="/mechanic/jobs" className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:border-blue-300">
                <p className="font-semibold">My jobs</p>
                <p className="mt-1 text-xs text-slate-500">Travel, arrive, repair, and complete accepted jobs.</p>
              </Link>
              <Link href="/mechanic/earnings" className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:border-blue-300">
                <p className="font-semibold">Earnings</p>
                <p className="mt-1 text-xs text-slate-500">Summary of completed work.</p>
              </Link>
              <Link href="/mechanic/payments" className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:border-blue-300">
                <p className="font-semibold">Payments</p>
                <p className="mt-1 text-xs text-slate-500">Payment status and receipts for your jobs.</p>
              </Link>
              <Link href="/mechanic/history" className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:border-blue-300">
                <p className="font-semibold">Past jobs</p>
                <p className="mt-1 text-xs text-slate-500">Completed, cancelled, and expired incidents.</p>
              </Link>
            </div>

            <ServiceProfileCard profile={profile.data} onEditArea={(area) => void editArea(area)} />
          </div>
        )}
      </main>
    </>
  );
}
