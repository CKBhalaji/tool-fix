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
    <Card className={selected ? "border-green-400 ring-2 ring-green-200" : ""}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-semibold text-slate-900">
            {offer.mechanic_name ?? `Mechanic #${offer.mechanic_id.slice(0, 8)}`}
          </p>
          <p className="mt-0.5 text-xs text-slate-500">
            ★ {offer.mechanic_rating?.toFixed(1) ?? "new"} · {offer.mechanic_completed_jobs} jobs
          </p>
        </div>
        <p className="text-lg font-bold text-slate-900">{formatMinor(offer.quoted_price_minor)}</p>
      </div>
      <dl className="mt-3 grid grid-cols-2 gap-2 text-xs text-slate-600">
        <div>
          <dt className="font-medium text-slate-400">ETA</dt>
          <dd>{offer.estimated_arrival_minutes} min</dd>
        </div>
        <div>
          <dt className="font-medium text-slate-400">Status</dt>
          <dd className="capitalize">{offer.status}</dd>
        </div>
      </dl>
      {offer.message ? (
        <p className="mt-3 rounded-lg bg-slate-50 p-2 text-xs text-slate-600">{offer.message}</p>
      ) : null}
      {onSelect ? (
        <button
          type="button"
          onClick={onSelect}
          disabled={offer.status !== "pending"}
          className="mt-4 w-full rounded-xl bg-blue-600 py-2 text-sm font-semibold text-white transition hover:bg-blue-700 disabled:opacity-50"
        >
          {offer.status === "pending" ? "Accept offer" : "Unavailable"}
        </button>
      ) : null}
    </Card>
  );
}
