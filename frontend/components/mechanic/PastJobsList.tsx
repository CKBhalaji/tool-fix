"use client";

import { Card, EmptyState, StatusBadge } from "@/components/ui";
import { formatMinor, type Job, type JobStatus } from "@/types";

const TERMINAL: JobStatus[] = [
  "completed",
  "cancelled",
  "expired",
  "failed",
  "no_mechanic_available",
];

/** Past incidents for the mechanic: their finished jobs with outcomes. */
export function PastJobsList({
  jobs,
  descriptions,
}: {
  jobs: Job[];
  descriptions: Record<string, string>;
}) {
  const past = jobs.filter((job) => TERMINAL.includes(job.status));

  if (past.length === 0) {
    return (
      <Card>
        <EmptyState message="No past jobs yet. Jobs that finish (completed, cancelled, expired) appear here." />
      </Card>
    );
  }

  return (
    <div className="grid gap-3">
      {past.map((job) => (
        <Card key={job.id} className="flex flex-wrap items-center justify-between gap-3">
          <div className="min-w-0">
            <StatusBadge status={job.status} />
            <p className="mt-1 truncate text-sm font-medium text-foreground">
              {descriptions[job.id] ?? "Breakdown request"}
            </p>
            <p className="text-xs text-muted">
              {new Date(job.updated_at).toLocaleString("en-IN")}
            </p>
          </div>
          <p className="text-sm font-semibold text-foreground">
            {job.final_amount_minor ? formatMinor(job.final_amount_minor) : "—"}
          </p>
        </Card>
      ))}
    </div>
  );
}
