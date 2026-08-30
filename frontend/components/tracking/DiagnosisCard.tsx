"use client";

import { Card } from "@/components/ui";
import type { Diagnosis, PriceEstimate } from "@/types";
import { formatMinor } from "@/types";

/** Advisory AI assessment + estimated range (display-only by design). */
export function DiagnosisCard({
  diagnosis,
  estimate,
}: {
  diagnosis: Diagnosis;
  estimate: PriceEstimate | null;
}) {
  return (
    <Card>
      <h2 className="font-semibold">AI assessment (advisory)</h2>
      <p className="mt-1 text-sm text-slate-600">
        Possible issue: <span className="font-medium">{diagnosis.possible_issue}</span> ·
        confidence {(diagnosis.confidence * 100).toFixed(0)}% · severity {diagnosis.severity}
      </p>
      {estimate ? (
        <p className="mt-1 text-sm text-slate-600">
          Estimated cost: {formatMinor(estimate.estimated_cost_min_minor)} –{" "}
          {formatMinor(estimate.estimated_cost_max_minor)}
        </p>
      ) : null}
      <p className="mt-2 rounded-lg bg-slate-50 p-2 text-xs text-slate-500">
        {diagnosis.reasoning_summary}
      </p>
    </Card>
  );
}
