"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { Card, PrimaryButton, StatusBadge } from "@/components/ui";
import { listMyJobs } from "@/services/jobs";
import { listVehicles } from "@/services/vehicles";
import { formatMinor, type Job, type Vehicle } from "@/types";

export default function CustomerDashboardPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const [jobs, setJobs] = useState<Job[]>([]);
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "customer")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  useEffect(() => {
    if (me?.user.role !== "customer") return;
    void listMyJobs().then(setJobs).catch(() => setJobs([]));
    void listVehicles().then(setVehicles).catch(() => setVehicles([]));
  }, [me]);

  return (
    <>
      <Header title="Customer" />
      <main className="mx-auto w-full max-w-5xl flex-1 px-4 py-6">
        <div className="mb-6 flex flex-col items-start justify-between gap-4 rounded-2xl bg-blue-600 p-6 text-white sm:flex-row sm:items-center">
          <div>
            <h1 className="text-2xl font-bold">Need roadside help?</h1>
            <p className="mt-1 text-sm text-blue-100">
              Share your location, describe the problem, and nearby mechanics will bid to help.
            </p>
          </div>
          <Link href="/customer/request">
            <PrimaryButton className="!bg-white !text-blue-700 hover:!bg-blue-50">
              Request assistance
            </PrimaryButton>
          </Link>
        </div>

        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-slate-500">
          Your vehicles
        </h2>
        <div className="mb-8 grid gap-3 sm:grid-cols-3">
          {vehicles.length === 0 ? (
            <Card>
              <p className="text-sm text-slate-500">No vehicles yet — you can add one when you request help.</p>
            </Card>
          ) : (
            vehicles.map((vehicle) => (
              <Card key={vehicle.id}>
                <p className="font-semibold capitalize">{vehicle.vehicle_kind}</p>
                <p className="text-xs text-slate-500">
                  {[vehicle.make, vehicle.model, vehicle.year].filter(Boolean).join(" ") || "—"}
                </p>
              </Card>
            ))
          )}
        </div>

        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-slate-500">
          Assistance history
        </h2>
        <div className="grid gap-3">
          {jobs.length === 0 ? (
            <Card>
              <p className="text-sm text-slate-500">No requests yet.</p>
            </Card>
          ) : (
            jobs.map((job) => (
              <Card key={job.id} className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <StatusBadge status={job.status} />
                  <p className="mt-1 text-xs text-slate-500">
                    Requested {new Date(job.created_at).toLocaleString("en-IN")}
                  </p>
                </div>
                <p className="text-sm font-semibold text-slate-900">
                  {job.final_amount_minor ? formatMinor(job.final_amount_minor) : "—"}
                </p>
                <div className="flex gap-2">
                  <Link
                    href={`/customer/offers?job_id=${job.id}`}
                    className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-50"
                  >
                    Offers
                  </Link>
                  <Link
                    href={`/customer/tracking?job_id=${job.id}`}
                    className="rounded-lg bg-slate-900 px-3 py-1.5 text-xs font-medium text-white hover:bg-slate-700"
                  >
                    Track
                  </Link>
                </div>
              </Card>
            ))
          )}
        </div>
      </main>
    </>
  );
}
