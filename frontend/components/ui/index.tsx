import { STATUS_LABELS, type JobStatus } from "@/types";

export function StatusBadge({ status }: { status: JobStatus }) {
  const styles: Record<JobStatus, string> = {
    created: "bg-slate-100 text-slate-700",
    analyzing: "bg-violet-100 text-violet-700",
    mechanics_searching: "bg-amber-100 text-amber-800",
    mechanics_notified: "bg-amber-100 text-amber-800",
    offers_received: "bg-sky-100 text-sky-800",
    mechanic_selected: "bg-blue-100 text-blue-800",
    mechanic_en_route: "bg-blue-100 text-blue-800",
    mechanic_arrived: "bg-indigo-100 text-indigo-800",
    repair_in_progress: "bg-orange-100 text-orange-800",
    repair_completed: "bg-emerald-100 text-emerald-800",
    payment_pending: "bg-yellow-100 text-yellow-800",
    completed: "bg-green-100 text-green-800",
    cancelled: "bg-slate-200 text-slate-600",
    expired: "bg-slate-200 text-slate-600",
    failed: "bg-red-100 text-red-700",
    no_mechanic_available: "bg-red-100 text-red-700",
  };
  return (
    <span className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${styles[status]}`}>
      {STATUS_LABELS[status]}
    </span>
  );
}

export function Card({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={`rounded-2xl border border-slate-200 bg-white p-5 shadow-sm ${className}`}>
      {children}
    </div>
  );
}

export function PrimaryButton({
  children,
  onClick,
  disabled,
  type = "button",
  className = "",
}: {
  children: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  type?: "button" | "submit";
  className?: string;
}) {
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={`rounded-xl bg-blue-600 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50 ${className}`}
    >
      {children}
    </button>
  );
}

export function SecondaryButton({
  children,
  onClick,
  disabled,
  className = "",
}: {
  children: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={`rounded-xl border border-slate-300 bg-white px-4 py-2.5 text-sm font-semibold text-slate-700 transition hover:bg-slate-50 disabled:opacity-50 ${className}`}
    >
      {children}
    </button>
  );
}
