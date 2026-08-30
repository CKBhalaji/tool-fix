"use client";

import { useCallback, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useGeolocation } from "@/hooks/useGeolocation";
import { Header } from "@/components/layout/Header";
import { NearbyRequestsFeed } from "@/components/mechanic/NearbyRequestsFeed";
import { nearbyRequests, pushLocation } from "@/services/mechanics";
import { createOffer } from "@/services/offers";
import { useAsync } from "@/hooks/useAsync";

export default function MechanicRequestsPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const { position, locateOnce } = useGeolocation(true);
  const [lastShare, setLastShare] = useState<string | null>(null);

  // Feed + location share: setState happens inside promise continuations.
  const feed = useAsync(
    () =>
      position
        ? nearbyRequests(position.latitude, position.longitude)
            .then((items) =>
              pushLocation(
                position.latitude,
                position.longitude,
                position.accuracy ?? undefined,
              ).then(() => {
                setLastShare(new Date().toLocaleTimeString("en-IN"));
                return items;
              }),
            )
        : Promise.resolve([]),
    [position],
  );

  if (!loading && (!me || me.user.role !== "mechanic")) {
    router.replace("/");
  }

  const submitBid = useCallback(
    async (jobId: string, priceRupees: number, etaMinutes: number, message: string) => {
      await createOffer(jobId, {
        quoted_price_minor: Math.round(priceRupees * 100),
        estimated_arrival_minutes: etaMinutes,
        message: message || undefined,
      });
      await feed.reload();
    },
    [feed],
  );

  return (
    <>
      <Header title="Nearby requests" />
      <main className="mx-auto w-full max-w-4xl flex-1 px-4 py-6">
        <NearbyRequestsFeed
          position={position}
          lastShare={lastShare}
          items={feed.data ?? []}
          error={feed.error}
          onSubmitBid={submitBid}
          onRefreshLocation={locateOnce}
        />
      </main>
    </>
  );
}
