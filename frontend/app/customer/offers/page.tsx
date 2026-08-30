"use client";

import { Suspense, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useAsync } from "@/hooks/useAsync";
import { Header } from "@/components/layout/Header";
import { OfferList } from "@/components/offers/OfferList";
import { ErrorText, StatusBadge } from "@/components/ui";
import { getJob } from "@/services/jobs";
import { listOffers, selectOffer } from "@/services/offers";

function OffersInner() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const params = useSearchParams();
  const jobId = params.get("job_id") ?? "";
  const [selecting, setSelecting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [reloadTick, setReloadTick] = useState(0);

  const offers = useAsync(
    () => (jobId ? listOffers(jobId) : Promise.resolve([])),
    [jobId, reloadTick],
  );
  const job = useAsync(
    () => (jobId ? getJob(jobId) : Promise.resolve(null)),
    [jobId, reloadTick],
  );

  if (!loading && (!me || me.user.role !== "customer")) {
    router.replace("/");
  }

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
    return <main className="flex flex-1 items-center justify-center text-sm text-muted">No job selected.</main>;
  }

  return (
    <>
      <Header title="Compare mechanics" />
      <main className="mx-auto w-full max-w-4xl flex-1 px-4 py-6">
        {job.data ? (
          <div className="mb-4 flex items-center gap-3">
            <StatusBadge status={job.data.status} />
            <span className="text-sm text-muted">
              {(offers.data ?? []).filter((offer) => offer.status === "pending").length} pending offer(s)
            </span>
          </div>
        ) : null}

        <ErrorText>{error}</ErrorText>

        <OfferList
          offers={offers.data ?? []}
          selecting={selecting}
          onSelect={(offerId) => void choose(offerId)}
          onRefresh={() => setReloadTick((tick) => tick + 1)}
        />

        <p className="mt-6 text-center text-xs text-muted">
          Offers come from independent mechanics — compare price, rating, and ETA before choosing.
        </p>
      </main>
    </>
  );
}

export default function OffersPage() {
  return (
    <Suspense fallback={<main className="flex flex-1 items-center justify-center text-sm text-muted">Loading…</main>}>
      <OffersInner />
    </Suspense>
  );
}
