"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import { Header } from "@/components/layout/Header";
import { PrimaryButton } from "@/components/ui";
import { AssistanceRequestList } from "@/components/customer/AssistanceRequestList";
import { VehicleCard } from "@/components/customer/VehicleCard";
import { listMyJobs } from "@/services/jobs";
import { listVehicles } from "@/services/vehicles";

export default function CustomerDashboardPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const jobs = useAsync(() => listMyJobs(), [me?.user.id]);
  const vehicles = useAsync(() => listVehicles(), [me?.user.id]);

  if (!loading && (!me || me.user.role !== "customer")) {
    router.replace("/");
  }

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
          {vehicles.data && vehicles.data.length > 0 ? (
            vehicles.data.map((vehicle) => (
              <VehicleCard key={vehicle.id} vehicle={vehicle} />
            ))
          ) : (
            <p className="text-sm text-slate-500">
              No vehicles yet — you can add one when you request help.
            </p>
          )}
        </div>

        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-slate-500">
          Assistance history
        </h2>
        <div className="mb-3 flex gap-2">
          <Link
            href="/customer/history"
            className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-50"
          >
            Past incidents
          </Link>
          <Link
            href="/customer/payments"
            className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-50"
          >
            Payment history
          </Link>
        </div>
        <AssistanceRequestList jobs={jobs.data ?? []} />
      </main>
    </>
  );
}
