"use client";

import { Card, EmptyState } from "@/components/ui";
import type { AdminAgentRun } from "@/services/admin";

/** AI-usage audit table: every agent run with provider, model, and result. */
export function AgentRunsTable({ runs }: { runs: AdminAgentRun[] }) {
  return (
    <Card className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="text-xs uppercase text-muted">
            <th className="py-2 pr-3">Created</th>
            <th className="py-2 pr-3">Kind</th>
            <th className="py-2 pr-3">Provider / model</th>
            <th className="py-2 pr-3">Status</th>
            <th className="py-2 pr-3">Latency</th>
            <th className="py-2">Problem</th>
          </tr>
        </thead>
        <tbody>
          {runs.map((run) => (
            <tr key={run.run_id} className="border-t border-border">
              <td className="py-2 pr-3 text-xs text-muted">
                {new Date(run.created_at).toLocaleString("en-IN")}
              </td>
              <td className="py-2 pr-3 capitalize">{run.kind.replace(/_/g, " ")}</td>
              <td className="py-2 pr-3 text-xs text-secondary">
                {run.provider}
                <br />
                <span className="text-muted">{run.model ?? "—"}</span>
              </td>
              <td className="py-2 pr-3">
                <span
                  className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${
                    run.status === "succeeded"
                      ? "bg-success-light text-success"
                      : "bg-error-light text-error"
                  }`}
                  title={run.error_message ?? undefined}
                >
                  {run.status}
                </span>
              </td>
              <td className="py-2 pr-3 text-xs text-muted">
                {run.latency_ms != null ? `${(run.latency_ms / 1000).toFixed(1)}s` : "—"}
              </td>
              <td className="max-w-xs py-2">
                <p className="truncate text-xs text-secondary" title={run.problem_description}>
                  {run.problem_description}
                </p>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <EmptyState message={runs.length === 0 ? "No AI runs recorded yet." : ""} />
    </Card>
  );
}
