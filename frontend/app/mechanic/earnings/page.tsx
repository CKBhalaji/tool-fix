"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { Card } from "@/components/ui";
import { listMyJobs } from "@/services/jobs";
import { formatMinor, type Job } from "@/types";

export default function MechanicEarningsPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const [jobs, setJobs] = useState<Job[]>([]);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "mechanic")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  useEffect(() => {
    if (me?.user.role !== "mechanic") return;
    void listMyJobs()
      .then((all) => setJobs(all.filter((job) => job.status === "completed")))
      .catch(() => setJobs([]));
  }, [me]);

  const totalMinor = jobs.reduce((sum, job) => sum + (job.final_amount_minor ?? 0), 0);

  return (
    <>
      <Header title="Earnings" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <Card className="mb-5">
          <p className="text-sm text-slate-500">Total from completed jobs</p>
          <p className="text-3xl font-bold text-slate-900">{formatMinor(totalMinor)}</p>
          <p className="mt-1 text-xs text-slate-500">{jobs.length} completed job(s)</p>
        </Card>
        <div className="grid gap-3">
          {jobs.map((job) => (
            <Card key={job.id} className="flex items-center justify-between">
              <span className="text-sm text-slate-600">
                {new Date(job.updated_at).toLocaleDateString("en-IN")}
              </span>
              <span className="font-semibold">{formatMinor(job.final_amount_minor)}</span>
            </Card>
          ))}
        </div>
      </main>
    </>
  );
}
