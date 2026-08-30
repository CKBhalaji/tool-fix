"use client";

import { Suspense, useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { DiagnosisCard } from "@/components/tracking/DiagnosisCard";
import { JobStatusPanel } from "@/components/tracking/JobStatusPanel";
import { TrackingMap } from "@/components/tracking/TrackingMap";
import { Card, Spinner } from "@/components/ui";
import { getBreakdown } from "@/services/breakdowns";
import { getJob, payJob, subscribeJobEvents } from "@/services/jobs";
import type { BreakdownDetail, Job } from "@/types";

function TrackingInner() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const params = useSearchParams();
  const jobId = params.get("job_id") ?? "";
  const [detail, setDetail] = useState<BreakdownDetail | null>(null);
  const [job, setJob] = useState<Job | null>(null);
  const [mechanicPosition, setMechanicPosition] = useState<{ latitude: number; longitude: number } | null>(null);
  const [paying, setPaying] = useState(false);

  if (!loading && (!me || me.user.role !== "customer")) {
    router.replace("/");
  }

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
    return <main className="flex flex-1 items-center justify-center text-sm text-muted">No job selected.</main>;
  }

  return (
    <>
      <Header title="Track assistance" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <div className="grid gap-5">
          <JobStatusPanel
            job={job}
            paying={paying}
            onPay={() => void pay()}
            onViewOffers={() => router.push(`/customer/offers?job_id=${job!.id}`)}
          />

          <TrackingMap
            breakdown={detail?.breakdown ?? null}
            mechanicPosition={mechanicPosition}
          />

          {detail?.diagnosis ? (
            <DiagnosisCard diagnosis={detail.diagnosis} estimate={detail.price_estimate} />
          ) : (
            <Card>
              <Spinner label="AI analysis is running — this page updates automatically." />
            </Card>
          )}
        </div>
      </main>
    </>
  );
}

export default function TrackingPage() {
  return (
    <Suspense fallback={<main className="flex flex-1 items-center justify-center text-sm text-muted">Loading…</main>}>
      <TrackingInner />
    </Suspense>
  );
}
