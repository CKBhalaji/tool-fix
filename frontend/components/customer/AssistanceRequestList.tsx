"use client";

import Link from "next/link";
import { Card, EmptyState, StatusBadge } from "@/components/ui";
import { formatMinor, type Job } from "@/types";

/** Customer-facing assistance history list. */
export function AssistanceRequestList({ jobs }: { jobs: Job[] }) {
  if (jobs.length === 0) {
    return (
      <Card>
        <EmptyState message="No requests yet." />
      </Card>
    );
  }

  return (
    <div className="grid gap-3">
      {jobs.map((job) => (
        <Card key={job.id} className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <StatusBadge status={job.status} />
            <p className="mt-1 text-xs text-slate-500">
              Requested {new Date(job.created_at).toLocaleString("en-IN")}
            </p>
          </div>
          <p className="text-sm font-semibold text-slate-900">
            {job.final_amount_minor ? formatMinor(job.final_amount_minor) : "—"}
          </p>
          <div className="flex gap-2">
            <Link
              href={`/customer/offers?job_id=${job.id}`}
              className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-50"
            >
              Offers
            </Link>
            <Link
              href={`/customer/tracking?job_id=${job.id}`}
              className="rounded-lg bg-slate-900 px-3 py-1.5 text-xs font-medium text-white hover:bg-slate-700"
            >
              Track
            </Link>
          </div>
        </Card>
      ))}
    </div>
  );
}
