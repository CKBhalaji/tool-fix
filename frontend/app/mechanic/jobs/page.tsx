"use client";

import { useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { MyJobsList } from "@/components/mechanic/MyJobsList";
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
  const [details, setDetails] = useState<Record<string, BreakdownDetail | null>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "mechanic")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  useEffect(() => {
    if (me?.user.role !== "mechanic") return;
    let ignore = false;
    void (async () => {
      try {
        const mine = await listMyJobs();
        if (ignore) return;
        setJobs(mine);
        const loaded: Record<string, BreakdownDetail | null> = {};
        for (const job of mine) {
          loaded[job.breakdown_id] = await getBreakdown(job.breakdown_id).catch(
            () => null,
          );
        }
        if (!ignore) setDetails(loaded);
      } catch (err) {
        if (!ignore) setError(err instanceof Error ? err.message : "Could not load jobs");
      }
    })();
    return () => {
      ignore = true;
    };
  }, [me]);

  const act = useCallback(
    async (key: string, jobId: string) => {
      setError(null);
      const action: Record<string, (id: string) => Promise<Job>> = {
        startTravel,
        markArrived,
        startRepair,
        completeRepair,
      };
      try {
        const updated = await action[key](jobId);
        setJobs((prev) => prev.map((job) => (job.id === jobId ? updated : job)));
      } catch (err) {
        setError(err instanceof Error ? err.message : "Action failed");
      }
    },
    [],
  );

  const actions = {
    startTravel: (id: string) => act("startTravel", id),
    markArrived: (id: string) => act("markArrived", id),
    startRepair: (id: string) => act("startRepair", id),
    completeRepair: (id: string) => act("completeRepair", id),
  };

  return (
    <>
      <Header title="My jobs" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <MyJobsList jobs={jobs} details={details} actions={actions} error={error} />
      </main>
    </>
  );
}
