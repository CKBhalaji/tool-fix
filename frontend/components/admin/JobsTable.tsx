"use client";

import { Card, EmptyState, StatusBadge } from "@/components/ui";
import { formatMinor } from "@/types";
import type { AdminJob } from "@/services/admin";

export function JobsTable({ jobs }: { jobs: AdminJob[] }) {
  return (
    <Card className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="text-xs uppercase text-muted">
            <th className="py-2 pr-3">Created</th>
            <th className="py-2 pr-3">Status</th>
            <th className="py-2 pr-3">Problem</th>
            <th className="py-2 pr-3">Customer</th>
            <th className="py-2 pr-3">Mechanic</th>
            <th className="py-2">Amount</th>
          </tr>
        </thead>
        <tbody>
          {jobs.map((job) => (
            <tr key={job.job_id} className="border-t border-border">
              <td className="py-2 pr-3 text-xs text-muted">
                {new Date(job.created_at).toLocaleString("en-IN")}
              </td>
              <td className="py-2 pr-3"><StatusBadge status={job.status} /></td>
              <td className="max-w-xs py-2 pr-3">
                <p className="truncate text-foreground" title={job.problem_description}>
                  {job.problem_description}
                </p>
                <p className="text-xs text-muted">
                  {job.latitude.toFixed(4)}, {job.longitude.toFixed(4)}
                </p>
              </td>
              <td className="py-2 pr-3 text-xs text-secondary">
                {job.customer_email ?? job.customer_name ?? "—"}
              </td>
              <td className="py-2 pr-3 text-xs text-secondary">{job.mechanic_name ?? "—"}</td>
              <td className="py-2 font-semibold">{formatMinor(job.final_amount_minor)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <EmptyState message={jobs.length === 0 ? "No jobs yet." : ""} />
    </Card>
  );
}
