"use client";

import { Card, PrimaryButton, SecondaryButton, StatusBadge } from "@/components/ui";
import { formatMinor, type Job } from "@/types";

/** Live job status header with the contextual customer actions. */
export function JobStatusPanel({
  job,
  paying,
  onPay,
  onViewOffers,
}: {
  job: Job | null;
  paying: boolean;
  onPay: () => void;
  onViewOffers: () => void;
}) {
  return (
    <Card className="flex flex-wrap items-center justify-between gap-3">
      <div>
        <p className="text-xs text-muted">Job status (live)</p>
        {job ? <StatusBadge status={job.status} /> : <p className="text-sm">Loading…</p>}
      </div>
      {job?.status === "repair_completed" && job.final_amount_minor ? (
        <PrimaryButton onClick={onPay} disabled={paying}>
          {paying ? "Processing…" : `Pay ${formatMinor(job.final_amount_minor)}`}
        </PrimaryButton>
      ) : null}
      {["created", "analyzing", "mechanics_searching", "mechanics_notified", "offers_received"].includes(
        job?.status ?? "",
      ) ? (
        <SecondaryButton onClick={onViewOffers}>View offers</SecondaryButton>
      ) : null}
    </Card>
  );
}
