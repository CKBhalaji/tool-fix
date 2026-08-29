// Types mirroring the Rust contract DTOs (toolfix-contracts).

export type UserRole = "customer" | "mechanic" | "admin";
export type UserStatus = "active" | "suspended" | "deleted";
export type OnboardingStatus = "pending" | "complete";

export interface User {
  id: string;
  firebase_uid: string;
  email: string | null;
  display_name: string | null;
  photo_url: string | null;
  phone: string | null;
  role: UserRole;
  status: UserStatus;
  onboarding_status: OnboardingStatus;
  created_at: string;
}

export interface AuthMe {
  user: User;
  mechanic_id: string | null;
}

export type VehicleKind =
  | "motorcycle"
  | "scooter"
  | "car"
  | "van"
  | "truck"
  | "other";

export interface Vehicle {
  id: string;
  owner_user_id: string;
  vehicle_kind: VehicleKind;
  make: string | null;
  model: string | null;
  year: number | null;
  registration_number: string | null;
  created_at: string;
}

export type JobStatus =
  | "created"
  | "analyzing"
  | "mechanics_searching"
  | "mechanics_notified"
  | "offers_received"
  | "mechanic_selected"
  | "mechanic_en_route"
  | "mechanic_arrived"
  | "repair_in_progress"
  | "repair_completed"
  | "payment_pending"
  | "completed"
  | "cancelled"
  | "expired"
  | "failed"
  | "no_mechanic_available";

export interface Job {
  id: string;
  breakdown_id: string;
  customer_user_id: string;
  vehicle_id: string;
  status: JobStatus;
  selected_mechanic_id: string | null;
  final_amount_minor: number | null;
  currency: string;
  offer_window_expires_at: string | null;
  created_at: string;
  updated_at: string;
}

export type Severity = "low" | "medium" | "high" | "critical";
export type RepairCategory =
  | "battery"
  | "tyre"
  | "engine"
  | "electrical"
  | "brakes"
  | "clutch"
  | "fuel"
  | "chain_drive"
  | "cooling"
  | "body_damage"
  | "lockout"
  | "towing"
  | "diagnostic"
  | "other";
export type EstimateSource = "agent" | "historical";

export interface Diagnosis {
  possible_issue: string;
  confidence: number;
  severity: Severity;
  recommended_service: string;
  repair_category: RepairCategory;
  estimated_cost_min_minor: number;
  estimated_cost_max_minor: number;
  requires_towing: boolean;
  reasoning_summary: string;
}

export interface PriceEstimate {
  repair_category: RepairCategory;
  estimated_cost_min_minor: number;
  estimated_cost_max_minor: number;
  currency: string;
  source: EstimateSource;
  notes: string | null;
  created_at: string;
}

export interface BreakdownDetail {
  breakdown: {
    id: string;
    latitude: number;
    longitude: number;
    address: string | null;
    problem_description: string;
    vehicle_symptoms: string[];
    created_at: string;
  };
  job: Job;
  diagnosis: Diagnosis | null;
  price_estimate: PriceEstimate | null;
  media: {
    id: string;
    breakdown_id: string;
    media_kind: "photo" | "video";
    storage_key: string;
    content_type: string | null;
    created_at: string;
  }[];
}

export type OfferStatus =
  | "pending"
  | "accepted"
  | "withdrawn"
  | "expired"
  | "rejected";

export interface Offer {
  id: string;
  job_id: string;
  mechanic_id: string;
  mechanic_name: string | null;
  mechanic_rating: number | null;
  mechanic_completed_jobs: number;
  quoted_price_minor: number;
  currency: string;
  estimated_arrival_minutes: number;
  message: string | null;
  status: OfferStatus;
  expires_at: string;
  created_at: string;
}

export interface MechanicProfile {
  mechanic_id: string;
  user: User;
  display_name: string | null;
  phone: string | null;
  city: string | null;
  service_area_km: number;
  supported_vehicle_kinds: VehicleKind[];
  repair_categories: RepairCategory[];
  experience_years: number | null;
  availability_status: "online" | "busy" | "offline";
  rating_average: number | null;
  completed_jobs: number;
  is_verified: boolean;
  current_latitude: number | null;
  current_longitude: number | null;
  location_updated_at: string | null;
}

export interface MechanicFeedItem {
  job_id: string;
  breakdown_id: string;
  problem_description: string;
  vehicle_symptoms: string[];
  latitude: number;
  longitude: number;
  address: string | null;
  distance_km: number;
  vehicle_kind: VehicleKind | null;
  diagnosis: Diagnosis | null;
  price_estimate: PriceEstimate | null;
  offer_window_expires_at: string | null;
  notified: boolean;
}

export interface JobStatusHistoryEntry {
  id: string;
  job_id: string;
  from_status: JobStatus | null;
  to_status: JobStatus;
  changed_by_user_id: string | null;
  reason: string | null;
  created_at: string;
}

export interface AppNotification {
  id: string;
  kind: string;
  channel: string;
  job_id: string | null;
  payload: Record<string, unknown>;
  status: string;
  created_at: string;
}

export interface JobEvent {
  job_id: string;
  kind: string;
  payload: Record<string, unknown>;
  at: string;
}

export function formatMinor(minor: number | null | undefined): string {
  if (minor == null) return "—";
  return `₹${(minor / 100).toLocaleString("en-IN", {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  })}`;
}

export const STATUS_LABELS: Record<JobStatus, string> = {
  created: "Created",
  analyzing: "AI analyzing",
  mechanics_searching: "Finding mechanics",
  mechanics_notified: "Mechanics notified",
  offers_received: "Offers received",
  mechanic_selected: "Mechanic selected",
  mechanic_en_route: "Mechanic on the way",
  mechanic_arrived: "Mechanic arrived",
  repair_in_progress: "Repair in progress",
  repair_completed: "Repair completed",
  payment_pending: "Payment pending",
  completed: "Completed",
  cancelled: "Cancelled",
  expired: "Expired",
  failed: "Failed",
  no_mechanic_available: "No mechanic available",
};
