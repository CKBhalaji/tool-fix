"use client";

import { useState } from "react";
import { MapView } from "@/components/map/MapView";
import { Card, ErrorText, PrimaryButton, SecondaryButton, TextInput } from "@/components/ui";
import { formatMinor, type MechanicFeedItem } from "@/types";

export interface FeedPosition {
  latitude: number;
  longitude: number;
  accuracy: number | null;
}

/** Nearby open breakdowns with the inline bid form, plus a location map. */
export function NearbyRequestsFeed({
  position,
  lastShare,
  items,
  error,
  onSubmitBid,
  onRefreshLocation,
}: {
  position: FeedPosition | null;
  lastShare: string | null;
  items: MechanicFeedItem[];
  error: string | null;
  onSubmitBid: (jobId: string, priceRupees: number, etaMinutes: number, message: string) => Promise<void>;
  onRefreshLocation: () => void;
}) {
  const [bidding, setBidding] = useState<string | null>(null);
  const [price, setPrice] = useState("");
  const [eta, setEta] = useState("20");
  const [message, setMessage] = useState("");
  const [bidError, setBidError] = useState<string | null>(null);

  async function submitBid(jobId: string) {
    setBidError(null);
    const rupees = Number(price);
    if (!Number.isFinite(rupees) || rupees < 50) {
      setBidError("Enter a price of at least ₹50");
      return;
    }
    try {
      await onSubmitBid(jobId, rupees, Number(eta) || 20, message);
      setBidding(null);
      setPrice("");
      setMessage("");
    } catch (err) {
      setBidError(err instanceof Error ? err.message : "Could not submit offer");
    }
  }

  return (
    <div className="grid gap-5">
      <Card>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-sm text-secondary">
            {position
              ? `Sharing location (±${Math.round(position.accuracy ?? 0)} m)${
                  lastShare ? ` · last ping ${lastShare}` : ""
                }`
              : "Enable location to receive nearby requests."}
          </p>
          <SecondaryButton onClick={onRefreshLocation}>Refresh my location</SecondaryButton>
        </div>
        {position ? (
          <MapView
            center={position}
            markers={[
              {
                id: "me",
                latitude: position.latitude,
                longitude: position.longitude,
                kind: "mechanic",
                label: "You",
              },
              ...items.slice(0, 10).map((item) => ({
                id: item.job_id,
                latitude: item.latitude,
                longitude: item.longitude,
                kind: "customer" as const,
                label: `${item.distance_km} km`,
              })),
            ]}
            className="mt-3 h-56 w-full overflow-hidden rounded-xl border border-border"
          />
        ) : null}
      </Card>

      <ErrorText>{error ?? bidError}</ErrorText>

      {items.length === 0 ? (
        <Card>
          <p className="text-sm text-muted">
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
                  <p className="mt-0.5 text-xs text-muted">
                    {item.distance_km} km away · {item.vehicle_kind ?? "unknown vehicle"}
                    {item.vehicle_symptoms.length ? ` · ${item.vehicle_symptoms.join(", ")}` : ""}
                  </p>
                </div>
                {item.price_estimate ? (
                  <p className="text-sm font-semibold text-foreground">
                    AI est. {formatMinor(item.price_estimate.estimated_cost_min_minor)} –{" "}
                    {formatMinor(item.price_estimate.estimated_cost_max_minor)}
                  </p>
                ) : null}
              </div>
              {item.diagnosis ? (
                <p className="mt-2 rounded-lg bg-background p-2 text-xs text-secondary">
                  {item.diagnosis.possible_issue} ({(item.diagnosis.confidence * 100).toFixed(0)}% confidence)
                </p>
              ) : null}

              {bidding === item.job_id ? (
                <div className="mt-3 grid gap-2 sm:grid-cols-4">
                  <TextInput value={price} onChange={setPrice} placeholder="Your price ₹" />
                  <TextInput value={eta} onChange={setEta} placeholder="ETA minutes" />
                  <TextInput
                    value={message}
                    onChange={setMessage}
                    placeholder="Note to customer (optional)"
                    className="sm:col-span-1"
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
    </div>
  );
}
