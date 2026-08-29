"use client";

import { Suspense, useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { Header } from "@/components/layout/Header";
import { OfferCard } from "@/components/offers";
import { Card, PrimaryButton, StatusBadge } from "@/components/ui";
import { getJob } from "@/services/jobs";
import { listOffers, selectOffer } from "@/services/offers";
import { type Job, type Offer } from "@/types";

function OffersInner() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const params = useSearchParams();
  const jobId = params.get("job_id") ?? "";
  const [job, setJob] = useState<Job | null>(null);
  const [offers, setOffers] = useState<Offer[]>([]);
  const [selecting, setSelecting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [reloadTick, setReloadTick] = useState(0);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "customer")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  // Load + refresh offers via promise callbacks (POST /select also refreshes
  // by bumping reloadTick from event handlers).
  useEffect(() => {
    if (!jobId) return;
    Promise.all([getJob(jobId), listOffers(jobId)])
      .then(([freshJob, freshOffers]) => {
        setJob(freshJob);
        setOffers(freshOffers);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : "Could not load offers");
      });
  }, [jobId, reloadTick]);

  async function choose(offerId: string) {
    setSelecting(offerId);
    setError(null);
    try {
      await selectOffer(offerId);
      router.push(`/customer/tracking?job_id=${jobId}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not accept the offer");
    } finally {
      setSelecting(null);
    }
  }

  if (!jobId) {
    return <main className="flex flex-1 items-center justify-center text-sm text-slate-500">No job selected.</main>;
  }

  const pending = offers.filter((offer) => offer.status === "pending");
  const decided = offers.filter((offer) => offer.status !== "pending");

  return (
    <>
      <Header title="Compare mechanics" />
      <main className="mx-auto w-full max-w-4xl flex-1 px-4 py-6">
        {job ? (
          <div className="mb-4 flex items-center gap-3">
            <StatusBadge status={job.status} />
            <span className="text-sm text-slate-500">
              {pending.length} pending {pending.length === 1 ? "offer" : "offers"}
            </span>
          </div>
        ) : null}

        {error ? <p className="mb-4 text-sm text-red-600">{error}</p> : null}

        {pending.length === 0 && decided.length === 0 ? (
          <Card>
            <p className="text-sm text-slate-500">
              No offers yet. Mechanics near you are being notified — refresh as bids arrive.
            </p>
            <div className="mt-3">
              <PrimaryButton onClick={() => setReloadTick((tick) => tick + 1)}>Refresh</PrimaryButton>
            </div>
          </Card>
        ) : (
          <div className="grid gap-4 sm:grid-cols-2">
            {pending.map((offer) => (
              <OfferCard
                key={offer.id}
                offer={offer}
                onSelect={() => void choose(offer.id)}
                selected={selecting === offer.id}
              />
            ))}
            {decided.map((offer) => (
              <OfferCard key={offer.id} offer={offer} />
            ))}
          </div>
        )}

        <p className="mt-6 text-center text-xs text-slate-400">
          Offers come from independent mechanics — compare price, rating, and ETA before choosing.
        </p>
      </main>
    </>
  );
}

export default function OffersPage() {
  return (
    <Suspense fallback={<main className="flex flex-1 items-center justify-center text-sm text-slate-500">Loading…</main>}>
      <OffersInner />
    </Suspense>
  );
}
