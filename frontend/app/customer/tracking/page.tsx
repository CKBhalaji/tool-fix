"use client";

import { Suspense, useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { MapView } from "@/components/map/MapView";
import { Card, PrimaryButton, SecondaryButton, StatusBadge } from "@/components/ui";
import { getBreakdown } from "@/services/breakdowns";
import { getJob, payJob, subscribeJobEvents } from "@/services/jobs";
import { formatMinor, type BreakdownDetail, type Job } from "@/types";

function TrackingInner() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const params = useSearchParams();
  const jobId = params.get("job_id") ?? "";
  const [detail, setDetail] = useState<BreakdownDetail | null>(null);
  const [job, setJob] = useState<Job | null>(null);
  const [mechanicPosition, setMechanicPosition] = useState<{ latitude: number; longitude: number } | null>(null);
  const [paying, setPaying] = useState(false);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "customer")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  // Initial load via promise callbacks (all setState in continuations).
  useEffect(() => {
    if (!jobId) return;
    Promise.all([
      getBreakdown(jobId).catch(() => null as unknown as BreakdownDetail),
      getJob(jobId).catch(() => null as unknown as Job),
    ]).then(([breakdown, freshJob]) => {
      if (breakdown?.job) {
        setDetail(breakdown);
        setJob(breakdown.job);
      } else if (freshJob) {
        setJob(freshJob);
        setDetail(null);
      }
    });
  }, [jobId]);

  // Live events: job state, offers, and mechanic position.
  useEffect(() => {
    if (!jobId) return;
    const dispose = subscribeJobEvents(jobId, (event) => {
      if (event.kind === "mechanic.location") {
        const latitude = Number(event.payload.latitude);
        const longitude = Number(event.payload.longitude);
        if (Number.isFinite(latitude) && Number.isFinite(longitude)) {
          setMechanicPosition({ latitude, longitude });
        }
      } else if (event.kind.startsWith("job.")) {
        void getJob(jobId).then(setJob).catch(() => undefined);
      }
    });
    return dispose;
  }, [jobId]);

  async function pay() {
    if (!job) return;
    setPaying(true);
    try {
      await payJob(job.id);
      await getJob(job.id).then(setJob);
    } finally {
      setPaying(false);
    }
  }

  if (!jobId) {
    return <main className="flex flex-1 items-center justify-center text-sm text-slate-500">No job selected.</main>;
  }

  const markers = [];
  if (detail?.breakdown) {
    markers.push({
      id: "breakdown",
      latitude: detail.breakdown.latitude,
      longitude: detail.breakdown.longitude,
      kind: "customer" as const,
      label: "You",
    });
  }
  if (mechanicPosition) {
    markers.push({ id: "mechanic", latitude: mechanicPosition.latitude, longitude: mechanicPosition.longitude, kind: "mechanic" as const, label: "Mechanic" });
  }

  return (
    <>
      <Header title="Track assistance" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <div className="grid gap-5">
          <Card className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-xs text-slate-500">Job status (live)</p>
              {job ? <StatusBadge status={job.status} /> : <p className="text-sm">Loading…</p>}
            </div>
            {job?.status === "repair_completed" && job.final_amount_minor ? (
              <PrimaryButton onClick={pay} disabled={paying}>
                {paying ? "Processing…" : `Pay ${formatMinor(job.final_amount_minor)}`}
              </PrimaryButton>
            ) : null}
            {["created", "analyzing", "mechanics_searching", "mechanics_notified", "offers_received"].includes(job?.status ?? "") ? (
              <SecondaryButton onClick={() => router.push(`/customer/offers?job_id=${job!.id}`)}>
                View offers
              </SecondaryButton>
            ) : null}
          </Card>

          <MapView
            center={detail?.breakdown ?? { latitude: 12.9716, longitude: 77.5946 }}
            markers={markers}
            className="h-72 w-full overflow-hidden rounded-xl border border-slate-200"
          />

          {detail?.diagnosis ? (
            <Card>
              <h2 className="font-semibold">AI assessment (advisory)</h2>
              <p className="mt-1 text-sm text-slate-600">
                Possible issue: <span className="font-medium">{detail.diagnosis.possible_issue}</span> ·
                confidence {(detail.diagnosis.confidence * 100).toFixed(0)}% · severity {detail.diagnosis.severity}
              </p>
              {detail.price_estimate ? (
                <p className="mt-1 text-sm text-slate-600">
                  Estimated cost: {formatMinor(detail.price_estimate.estimated_cost_min_minor)} –{" "}
                  {formatMinor(detail.price_estimate.estimated_cost_max_minor)}
                </p>
              ) : null}
              <p className="mt-2 rounded-lg bg-slate-50 p-2 text-xs text-slate-500">
                {detail.diagnosis.reasoning_summary}
              </p>
            </Card>
          ) : (
            <Card>
              <p className="text-sm text-slate-500">AI analysis is running — this page updates automatically.</p>
            </Card>
          )}
        </div>
      </main>
    </>
  );
}

export default function TrackingPage() {
  return (
    <Suspense fallback={<main className="flex flex-1 items-center justify-center text-sm text-slate-500">Loading…</main>}>
      <TrackingInner />
    </Suspense>
  );
}
