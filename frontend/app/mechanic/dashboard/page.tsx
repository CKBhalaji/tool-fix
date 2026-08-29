"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { Card, PrimaryButton, SecondaryButton } from "@/components/ui";
import { getMechanicProfile, setAvailability } from "@/services/mechanics";
import type { MechanicProfile } from "@/types";

export default function MechanicDashboardPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const [profile, setProfile] = useState<MechanicProfile | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "mechanic")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  useEffect(() => {
    if (me?.user.role !== "mechanic") return;
    void getMechanicProfile().then(setProfile).catch((err) => {
      // Profile may not exist yet if onboarding created it empty; surface a
      // friendly message via the profile form below.
      setError(err instanceof Error ? err.message : "Could not load profile");
    });
  }, [me]);

  async function toggleAvailability() {
    if (!profile) return;
    setBusy(true);
    try {
      const next =
        profile.availability_status === "offline" ? "online" : "offline";
      const updated = await setAvailability(next);
      setProfile(updated);
    } finally {
      setBusy(false);
    }
  }

  const online = profile?.availability_status !== "offline";

  return (
    <>
      <Header title="Mechanic" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <div className="grid gap-5">
          <Card>
            <div className="flex flex-wrap items-center justify-between gap-4">
              <div>
                <p className="text-sm font-semibold text-slate-900">
                  {profile?.display_name ?? me?.user.display_name ?? "Mechanic"}
                </p>
                <p className="text-xs text-slate-500">
                  {profile?.completed_jobs ?? 0} completed jobs ·{" "}
                  {profile?.rating_average ? `★ ${profile.rating_average.toFixed(1)}` : "not yet rated"}
                  {profile?.is_verified ? "" : " · verification pending"}
                </p>
              </div>
              <div className="flex items-center gap-3">
                <span
                  className={`inline-block rounded-full px-3 py-1 text-xs font-semibold ${
                    online ? "bg-green-100 text-green-700" : "bg-slate-200 text-slate-600"
                  }`}
                >
                  {profile?.availability_status ?? "offline"}
                </span>
                <PrimaryButton onClick={toggleAvailability} disabled={busy || !profile}>
                  {online ? "Go offline" : "Go online"}
                </PrimaryButton>
              </div>
            </div>
            {profile?.repair_categories.length ? (
              <p className="mt-3 text-xs text-slate-500">
                Services: {profile.repair_categories.join(", ")}
              </p>
            ) : (
              <p className="mt-3 text-xs text-amber-600">
                Set your services and vehicle types below to appear in matching.
              </p>
            )}
          </Card>

          {error ? <p className="text-sm text-red-600">{error}</p> : null}

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
          </div>

          <Card>
            <h2 className="mb-2 font-semibold">Service profile</h2>
            <p className="text-xs text-slate-500">
              Service area: {profile?.service_area_km ?? 10} km around your shared location.
            </p>
            <div className="mt-3">
              <SecondaryButton
                onClick={() => {
                  const area = window.prompt("Service area radius in km", String(profile?.service_area_km ?? 10));
                  if (!area) return;
                  void (async () => {
                    const { updateMechanicProfile } = await import("@/services/mechanics");
                    const updated = await updateMechanicProfile({ service_area_km: Number(area) });
                    setProfile(updated);
                  })();
                }}
              >
                Edit service area
              </SecondaryButton>
            </div>
          </Card>
        </div>
      </main>
    </>
  );
}
