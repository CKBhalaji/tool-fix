"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { Card, PrimaryButton, StatusBadge } from "@/components/ui";
import { getBreakdown } from "@/services/breakdowns";
import {
  completeRepair,
  listMyJobs,
  markArrived,
  startRepair,
  startTravel,
} from "@/services/jobs";
import type { BreakdownDetail, Job } from "@/types";

export default function MechanicJobsPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const [jobs, setJobs] = useState<Job[]>([]);
  const [details, setDetails] = useState<Record<string, BreakdownDetail>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "mechanic")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  useEffect(() => {
    if (me?.user.role !== "mechanic") return;
    void (async () => {
      try {
        const mine = await listMyJobs();
        setJobs(mine);
        const loaded: Record<string, BreakdownDetail> = {};
        for (const job of mine) {
          loaded[job.breakdown_id] = await getBreakdown(job.breakdown_id).catch(() => null as unknown as BreakdownDetail);
        }
        setDetails(loaded);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Could not load jobs");
      }
    })();
  }, [me]);

  async function act(jobId: string, action: () => Promise<Job>) {
    setError(null);
    try {
      const updated = await action();
      setJobs((prev) => prev.map((job) => (job.id === jobId ? updated : job)));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Action failed");
    }
  }

  return (
    <>
      <Header title="My jobs" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        {error ? <p className="mb-4 text-sm text-red-600">{error}</p> : null}
        {jobs.length === 0 ? (
          <Card>
            <p className="text-sm text-slate-500">No accepted jobs yet. Offers you win will appear here.</p>
          </Card>
        ) : (
          <div className="grid gap-4">
            {jobs.map((job) => {
              const detail = details[job.breakdown_id];
              const description = detail?.breakdown.problem_description ?? "Breakdown request";
              return (
                <Card key={job.id}>
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <StatusBadge status={job.status} />
                      <p className="mt-1 font-semibold">{description}</p>
                      {detail?.breakdown ? (
                        <p className="text-xs text-slate-500">
                          {detail.breakdown.latitude.toFixed(4)}, {detail.breakdown.longitude.toFixed(4)}
                          {detail.breakdown.address ? ` · ${detail.breakdown.address}` : ""}
                        </p>
                      ) : null}
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {job.status === "mechanic_selected" ? (
                        <PrimaryButton onClick={() => void act(job.id, () => startTravel(job.id))}>
                          Start travelling
                        </PrimaryButton>
                      ) : null}
                      {job.status === "mechanic_en_route" ? (
                        <PrimaryButton onClick={() => void act(job.id, () => markArrived(job.id))}>
                          Mark arrived
                        </PrimaryButton>
                      ) : null}
                      {job.status === "mechanic_arrived" ? (
                        <PrimaryButton onClick={() => void act(job.id, () => startRepair(job.id))}>
                          Start repair
                        </PrimaryButton>
                      ) : null}
                      {job.status === "repair_in_progress" ? (
                        <PrimaryButton onClick={() => void act(job.id, () => completeRepair(job.id))}>
                          Complete repair
                        </PrimaryButton>
                      ) : null}
                    </div>
                  </div>
                </Card>
              );
            })}
          </div>
        )}
      </main>
    </>
  );
}
