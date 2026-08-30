"use client";

import { Card, EmptyState } from "@/components/ui";
import { formatMinor, type Job } from "@/types";

/** Completed-job earnings summary. */
export function EarningsSummary({ jobs }: { jobs: Job[] }) {
  const totalMinor = jobs.reduce((sum, job) => sum + (job.final_amount_minor ?? 0), 0);

  return (
    <div className="grid gap-3">
      <Card className="mb-5">
        <p className="text-sm text-muted">Total from completed jobs</p>
        <p className="text-3xl font-bold text-foreground">{formatMinor(totalMinor)}</p>
        <p className="mt-1 text-xs text-muted">{jobs.length} completed job(s)</p>
      </Card>
      {jobs.map((job) => (
        <Card key={job.id} className="flex items-center justify-between">
          <span className="text-sm text-secondary">
            {new Date(job.updated_at).toLocaleDateString("en-IN")}
          </span>
          <span className="font-semibold">{formatMinor(job.final_amount_minor)}</span>
        </Card>
      ))}
      <EmptyState message={jobs.length === 0 ? "No completed jobs yet." : ""} />
    </div>
  );
}
