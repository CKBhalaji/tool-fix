import { Card } from "@/components/ui";
import { formatMinor, type Offer } from "@/types";

export function OfferCard({
  offer,
  selected,
  onSelect,
}: {
  offer: Offer;
  selected?: boolean;
  onSelect?: () => void;
}) {
  return (
    <Card className={selected ? "border-success ring-2 ring-success/30" : ""}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-semibold text-foreground">
            {offer.mechanic_name ?? `Mechanic #${offer.mechanic_id.slice(0, 8)}`}
          </p>
          <p className="mt-0.5 text-xs text-muted">
            ★ {offer.mechanic_rating?.toFixed(1) ?? "new"} · {offer.mechanic_completed_jobs} jobs
          </p>
        </div>
        <p className="text-lg font-bold text-accent">{formatMinor(offer.quoted_price_minor)}</p>
      </div>
      <dl className="mt-3 grid grid-cols-2 gap-2 text-xs text-secondary">
        <div>
          <dt className="font-medium text-muted">ETA</dt>
          <dd>{offer.estimated_arrival_minutes} min</dd>
        </div>
        <div>
          <dt className="font-medium text-muted">Status</dt>
          <dd className="capitalize">{offer.status}</dd>
        </div>
      </dl>
      {offer.message ? (
        <p className="mt-3 rounded-lg bg-background p-2 text-xs text-secondary">{offer.message}</p>
      ) : null}
      {onSelect ? (
        <button
          type="button"
          onClick={onSelect}
          disabled={offer.status !== "pending"}
          className="mt-4 w-full rounded-xl bg-primary py-2 text-sm font-semibold text-white transition hover:bg-primary-hover disabled:opacity-50"
        >
          {offer.status === "pending" ? "Accept offer" : "Unavailable"}
        </button>
      ) : null}
    </Card>
  );
}
