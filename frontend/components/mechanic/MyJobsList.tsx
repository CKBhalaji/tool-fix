"use client";

import { Card, EmptyState, PrimaryButton, StatusBadge } from "@/components/ui";
import type { BreakdownDetail, Job } from "@/types";

export interface JobAction {
  label: string;
  run: (jobId: string) => Promise<void>;
}

/** The mechanic's accepted jobs with state-machine-guarded actions. */
export function MyJobsList({
  jobs,
  details,
  actions,
  error,
}: {
  jobs: Job[];
  details: Record<string, BreakdownDetail | null>;
  actions: Record<string, (jobId: string) => Promise<void>>;
  error: string | null;
}) {
  if (jobs.length === 0) {
    return (
      <Card>
        <EmptyState message="No accepted jobs yet. Offers you win will appear here." />
      </Card>
    );
  }

  return (
    <div className="grid gap-4">
      {error ? <p className="text-sm text-red-600">{error}</p> : null}
      {jobs.map((job) => {
        const detail = details[job.breakdown_id];
        const description = detail?.breakdown.problem_description ?? "Breakdown request";
        const action = NEXT_ACTION[job.status];
        return (
          <Card key={job.id}>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <StatusBadge status={job.status} />
                <p className="mt-1 font-semibold">{description}</p>
                {detail?.breakdown ? (
                  <p className="text-xs text-slate-500">
                    {detail.breakdown.latitude.toFixed(4)}, {detail.breakdown.longitude.toFixed(4)}
                    {detail.breakdown.address ? ` · ${detail.breakdown.address}` : ""}
                  </p>
                ) : null}
              </div>
              {action ? (
                <PrimaryButton onClick={() => void actions[action.key](job.id)}>
                  {action.label}
                </PrimaryButton>
              ) : null}
            </div>
          </Card>
        );
      })}
    </div>
  );
}

const NEXT_ACTION: Record<string, { key: string; label: string } | undefined> = {
  mechanic_selected: { key: "startTravel", label: "Start travelling" },
  mechanic_en_route: { key: "markArrived", label: "Mark arrived" },
  mechanic_arrived: { key: "startRepair", label: "Start repair" },
  repair_in_progress: { key: "completeRepair", label: "Complete repair" },
};
