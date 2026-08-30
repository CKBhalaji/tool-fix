"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import { Header } from "@/components/layout/Header";
import { PastJobsList } from "@/components/mechanic/PastJobsList";
import { getBreakdown } from "@/services/breakdowns";
import { listMyJobs } from "@/services/jobs";
import type { BreakdownDetail } from "@/types";

export default function MechanicHistoryPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const jobs = useAsync(
    () => (me?.user.role === "mechanic" ? listMyJobs() : Promise.resolve([])),
    [me?.user.id],
  );
  const [descriptions, setDescriptions] = useState<Record<string, string>>({});

  useEffect(() => {
    if (!jobs.data) return;
    let ignore = false;
    void (async () => {
      const map: Record<string, string> = {};
      for (const job of jobs.data ?? []) {
        const detail: BreakdownDetail | null = await getBreakdown(job.breakdown_id).catch(() => null);
        map[job.id] = detail?.breakdown.problem_description ?? "Breakdown request";
      }
      if (!ignore) setDescriptions(map);
    })();
    return () => {
      ignore = true;
    };
  }, [jobs.data]);

  if (!loading && (!me || me.user.role !== "mechanic")) {
    router.replace("/");
  }

  return (
    <>
      <Header title="Past jobs" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <PastJobsList jobs={jobs.data ?? []} descriptions={descriptions} />
      </main>
    </>
  );
}
