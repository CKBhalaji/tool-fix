import { STATUS_LABELS, type JobStatus } from "@/types";

interface ChipStyle {
  chip: string;
  dot: string;
}

/**
 * Status chip: color + dot + label (never color alone — accessibility).
 * Semantic mapping per the design spec:
 *   blue → in progress, amber → waiting/attention, green → success,
 *   red → failure, neutral → finished without outcome.
 */
const STYLES: Record<JobStatus, ChipStyle> = {
  created: { chip: "bg-surface-secondary text-secondary", dot: "bg-muted" },
  analyzing: { chip: "bg-info-light text-info", dot: "bg-info" },
  mechanics_searching: { chip: "bg-warning-light text-warning", dot: "bg-accent" },
  mechanics_notified: { chip: "bg-warning-light text-warning", dot: "bg-accent" },
  offers_received: { chip: "bg-info-light text-info", dot: "bg-info" },
  mechanic_selected: { chip: "bg-primary-light text-primary", dot: "bg-primary" },
  mechanic_en_route: { chip: "bg-primary-light text-primary", dot: "bg-primary" },
  mechanic_arrived: { chip: "bg-info-light text-info", dot: "bg-info" },
  repair_in_progress: { chip: "bg-warning-light text-warning", dot: "bg-accent" },
  repair_completed: { chip: "bg-success-light text-success", dot: "bg-success" },
  payment_pending: { chip: "bg-warning-light text-warning", dot: "bg-accent" },
  completed: { chip: "bg-success-light text-success", dot: "bg-success" },
  cancelled: { chip: "bg-surface-secondary text-muted", dot: "bg-muted" },
  expired: { chip: "bg-surface-secondary text-muted", dot: "bg-muted" },
  failed: { chip: "bg-error-light text-error", dot: "bg-error" },
  no_mechanic_available: { chip: "bg-error-light text-error", dot: "bg-error" },
};

export function StatusBadge({ status }: { status: JobStatus }) {
  const style = STYLES[status];
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium ${style.chip}`}
    >
      <span className={`h-1.5 w-1.5 rounded-full ${style.dot}`} aria-hidden="true" />
      {STATUS_LABELS[status]}
    </span>
  );
}

export function Card({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={`rounded-2xl border border-border bg-surface p-5 shadow-sm ${className}`}>
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
      className={`rounded-xl bg-primary px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-primary-hover focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 ${className}`}
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
      className={`rounded-xl border border-border bg-surface px-4 py-2.5 text-sm font-semibold text-foreground transition hover:bg-surface-secondary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:opacity-50 ${className}`}
    >
      {children}
    </button>
  );
}

/** Amber — only for attention-critical actions (e.g. emergency request). */
export function AccentButton({
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
      className={`rounded-xl bg-accent px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-accent-hover focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:opacity-50 ${className}`}
    >
      {children}
    </button>
  );
}

export function DestructiveButton({
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
      className={`rounded-xl bg-error px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-error/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-error focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:opacity-50 ${className}`}
    >
      {children}
    </button>
  );
}

export function StatCard({ label, value }: { label: string; value: string | number }) {
  return (
    <Card>
      <p className="text-xs font-medium uppercase tracking-wide text-muted">{label}</p>
      <p className="mt-1 text-2xl font-bold text-foreground">{value}</p>
    </Card>
  );
}

export function ErrorText({ children }: { children: React.ReactNode }) {
  if (!children) return null;
  return <p className="text-sm text-error">{children}</p>;
}

export function Spinner({ label = "Loading…" }: { label?: string }) {
  return (
    <p className="flex items-center gap-2 text-sm text-muted">
      <span className="h-3 w-3 animate-spin rounded-full border-2 border-border border-t-primary" aria-hidden="true" />
      {label}
    </p>
  );
}

export function EmptyState({ message }: { message: string }) {
  if (!message) return null;
  return <p className="py-3 text-sm text-muted">{message}</p>;
}

export function TextInput({
  value,
  onChange,
  placeholder,
  type = "text",
  className = "",
}: {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: string;
  className?: string;
}) {
  return (
    <input
      type={type}
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
      className={`rounded-xl border border-border bg-surface px-3 py-2.5 text-sm text-foreground placeholder:text-muted transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary ${className}`}
    />
  );
}
