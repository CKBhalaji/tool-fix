"use client";

import Link from "next/link";
import { Card, EmptyState, StatusBadge } from "@/components/ui";
import { formatMinor, type Job } from "@/types";

const PAST_STATUSES = ["completed", "cancelled", "expired", "failed", "no_mechanic_available"];

/** Past incidents: the customer's finished (terminal-state) requests. */
export function PastIncidents({ jobs }: { jobs: Job[] }) {
  const past = jobs.filter((job) => PAST_STATUSES.includes(job.status));

  if (past.length === 0) {
    return (
      <Card>
        <EmptyState message="No past incidents yet. Requests that finish (completed, cancelled, expired) appear here." />
      </Card>
    );
  }

  return (
    <div className="grid gap-3">
      {past.map((job) => (
        <Card key={job.id} className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <StatusBadge status={job.status} />
            <p className="mt-1 text-xs text-muted">
              {new Date(job.created_at).toLocaleString("en-IN")}
            </p>
          </div>
          <p className="text-sm font-semibold text-foreground">
            {job.final_amount_minor ? formatMinor(job.final_amount_minor) : "—"}
          </p>
          <Link
            href={`/customer/tracking?job_id=${job.id}`}
            className="rounded-lg border border-border px-3 py-1.5 text-xs font-medium text-secondary hover:bg-surface-secondary"
          >
            Details
          </Link>
        </Card>
      ))}
    </div>
  );
}
