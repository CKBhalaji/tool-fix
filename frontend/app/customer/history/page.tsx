"use client";

import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import { Header } from "@/components/layout/Header";
import { PastIncidents } from "@/components/customer/PastIncidents";
import { listMyJobs } from "@/services/jobs";

export default function CustomerHistoryPage() {
  const { me } = useAuth();
  const jobs = useAsync(() => listMyJobs(), [me?.user.id]);

  return (
    <>
      <Header title="Past incidents" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        {jobs.error ? <p className="mb-3 text-sm text-red-600">{jobs.error}</p> : null}
        <PastIncidents jobs={jobs.data ?? []} />
      </main>
    </>
  );
}
