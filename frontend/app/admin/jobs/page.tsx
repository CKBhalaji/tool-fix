"use client";

import { JobsTable } from "@/components/admin/JobsTable";
import { useAsync } from "@/hooks/useAsync";
import { listJobs } from "@/services/admin";

export default function AdminJobsPage() {
  const { data, error } = useAsync(listJobs, []);

  return (
    <div>
      {error ? <p className="mb-3 text-sm text-red-600">{error}</p> : null}
      <JobsTable jobs={data ?? []} />
    </div>
  );
}
