"use client";

import { Card, PrimaryButton } from "@/components/ui";
import { OfferCard } from "@/components/offers";
import type { Offer } from "@/types";

/** Offer comparison grid: pending offers selectable, decided ones read-only. */
export function OfferList({
  offers,
  selecting,
  onSelect,
  onRefresh,
}: {
  offers: Offer[];
  selecting: string | null;
  onSelect: (offerId: string) => void;
  onRefresh: () => void;
}) {
  const pending = offers.filter((offer) => offer.status === "pending");
  const decided = offers.filter((offer) => offer.status !== "pending");

  if (pending.length === 0 && decided.length === 0) {
    return (
      <Card>
        <p className="text-sm text-slate-500">
          No offers yet. Mechanics near you are being notified — refresh as bids arrive.
        </p>
        <div className="mt-3">
          <PrimaryButton onClick={onRefresh}>Refresh</PrimaryButton>
        </div>
      </Card>
    );
  }

  return (
    <div className="grid gap-4 sm:grid-cols-2">
      {pending.map((offer) => (
        <OfferCard
          key={offer.id}
          offer={offer}
          onSelect={() => onSelect(offer.id)}
          selected={selecting === offer.id}
        />
      ))}
      {decided.map((offer) => (
        <OfferCard key={offer.id} offer={offer} />
      ))}
    </div>
  );
}
