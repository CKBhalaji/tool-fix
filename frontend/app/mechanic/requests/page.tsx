"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useGeolocation } from "@/hooks/useGeolocation";
import { Header } from "@/components/layout/Header";
import { MapView } from "@/components/map/MapView";
import { Card, PrimaryButton, SecondaryButton } from "@/components/ui";
import { nearbyRequests, pushLocation } from "@/services/mechanics";
import { createOffer } from "@/services/offers";
import { formatMinor, type MechanicFeedItem } from "@/types";

export default function MechanicRequestsPage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const { position, locateOnce } = useGeolocation(true);
  const [items, setItems] = useState<MechanicFeedItem[]>([]);
  const [bidding, setBidding] = useState<string | null>(null);
  const [price, setPrice] = useState("");
  const [eta, setEta] = useState("20");
  const [message, setMessage] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [lastShare, setLastShare] = useState<string | null>(null);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "mechanic")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  // Feed + location share: setState happens inside promise continuations.
  useEffect(() => {
    if (!position) return;
    nearbyRequests(position.latitude, position.longitude)
      .then((feed) => {
        setItems(feed);
        return pushLocation(
          position.latitude,
          position.longitude,
          position.accuracy ?? undefined,
        ).then(() => setLastShare(new Date().toLocaleTimeString("en-IN")));
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : "Could not load nearby requests");
      });
  }, [position]);

  async function submitBid(jobId: string) {
    setError(null);
    try {
      const rupees = Number(price);
      if (!Number.isFinite(rupees) || rupees < 50) {
        throw new Error("Enter a price of at least ₹50");
      }
      await createOffer(jobId, {
        quoted_price_minor: Math.round(rupees * 100),
        estimated_arrival_minutes: Number(eta) || 20,
        message: message || undefined,
      });
      setBidding(null);
      setPrice("");
      setMessage("");
      setItems((prev) => prev.filter((item) => item.job_id !== jobId));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not submit offer");
    }
  }

  return (
    <>
      <Header title="Nearby requests" />
      <main className="mx-auto w-full max-w-4xl flex-1 px-4 py-6">
        <Card className="mb-5">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="text-sm text-slate-600">
              {position
                ? `Sharing location (±${Math.round(position.accuracy ?? 0)} m)${
                    lastShare ? ` · last ping ${lastShare}` : ""
                  }`
                : "Enable location to receive nearby requests."}
            </p>
            <SecondaryButton onClick={locateOnce}>Refresh my location</SecondaryButton>
          </div>
          {position ? (
            <MapView
              center={position}
              markers={[
                { id: "me", latitude: position.latitude, longitude: position.longitude, kind: "mechanic", label: "You" },
                ...items.slice(0, 10).map((item) => ({
                  id: item.job_id,
                  latitude: item.latitude,
                  longitude: item.longitude,
                  kind: "customer" as const,
                  label: `${item.distance_km} km`,
                })),
              ]}
              className="mt-3 h-56 w-full overflow-hidden rounded-xl border border-slate-200"
            />
          ) : null}
        </Card>

        {error ? <p className="mb-4 text-sm text-red-600">{error}</p> : null}

        {items.length === 0 ? (
          <Card>
            <p className="text-sm text-slate-500">
              No open requests in your area right now. Stay online — new breakdowns appear here.
            </p>
          </Card>
        ) : (
          <div className="grid gap-4">
            {items.map((item) => (
              <Card key={item.job_id}>
                <div className="flex flex-wrap items-start justify-between gap-2">
                  <div>
                    <p className="font-semibold">{item.problem_description}</p>
                    <p className="mt-0.5 text-xs text-slate-500">
                      {item.distance_km} km away · {item.vehicle_kind ?? "unknown vehicle"}
                      {item.vehicle_symptoms.length ? ` · ${item.vehicle_symptoms.join(", ")}` : ""}
                    </p>
                  </div>
                  {item.price_estimate ? (
                    <p className="text-sm font-semibold text-slate-700">
                      AI est. {formatMinor(item.price_estimate.estimated_cost_min_minor)} –{" "}
                      {formatMinor(item.price_estimate.estimated_cost_max_minor)}
                    </p>
                  ) : null}
                </div>
                {item.diagnosis ? (
                  <p className="mt-2 rounded-lg bg-slate-50 p-2 text-xs text-slate-600">
                    {item.diagnosis.possible_issue} ({(item.diagnosis.confidence * 100).toFixed(0)}% confidence)
                  </p>
                ) : null}

                {bidding === item.job_id ? (
                  <div className="mt-3 grid gap-2 sm:grid-cols-4">
                    <input
                      value={price}
                      onChange={(event) => setPrice(event.target.value)}
                      placeholder="Your price ₹"
                      inputMode="numeric"
                      className="rounded-xl border border-slate-300 px-3 py-2 text-sm"
                    />
                    <input
                      value={eta}
                      onChange={(event) => setEta(event.target.value)}
                      placeholder="ETA minutes"
                      inputMode="numeric"
                      className="rounded-xl border border-slate-300 px-3 py-2 text-sm"
                    />
                    <input
                      value={message}
                      onChange={(event) => setMessage(event.target.value)}
                      placeholder="Note to customer (optional)"
                      className="rounded-xl border border-slate-300 px-3 py-2 text-sm sm:col-span-1"
                    />
                    <div className="flex gap-2">
                      <PrimaryButton onClick={() => void submitBid(item.job_id)}>Bid</PrimaryButton>
                      <SecondaryButton onClick={() => setBidding(null)}>Cancel</SecondaryButton>
                    </div>
                  </div>
                ) : (
                  <div className="mt-3">
                    <PrimaryButton onClick={() => setBidding(item.job_id)}>Submit offer</PrimaryButton>
                  </div>
                )}
              </Card>
            ))}
          </div>
        )}
      </main>
    </>
  );
}
