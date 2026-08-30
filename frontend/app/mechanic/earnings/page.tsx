"use client";

import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import Link from "next/link";
import { Header } from "@/components/layout/Header";
import { EarningsSummary } from "@/components/mechanic/EarningsSummary";
import { listMyJobs } from "@/services/jobs";
import type { Job } from "@/types";

export default function MechanicEarningsPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const jobs = useAsync(
    () =>
      listMyJobs().then((all) => all.filter((job) => job.status === "completed")),
    [me?.user.id],
  );

  if (!loading && (!me || me.user.role !== "mechanic")) {
    router.replace("/");
  }

  return (
    <>
      <Header title="Earnings" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <EarningsSummary jobs={(jobs.data ?? []) as Job[]} />
        <div className="mt-4">
          <Link href="/mechanic/payments" className="text-sm font-medium text-primary hover:underline">
            View payment records →
          </Link>
        </div>
      </main>
    </>
  );
}
