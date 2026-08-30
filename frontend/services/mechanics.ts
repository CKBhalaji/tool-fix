import { api } from "./api";
import type {
  MechanicFeedItem,
  MechanicProfile,
  RepairCategory,
  VehicleKind,
} from "@/types";

export async function getMechanicProfile(): Promise<MechanicProfile> {
  return api<MechanicProfile>("/api/v1/mechanics/me");
}

export async function setAvailability(
  status: MechanicProfile["availability_status"],
): Promise<MechanicProfile> {
  return api<MechanicProfile>("/api/v1/mechanics/availability", {
    method: "POST",
    body: { status },
  });
}

export async function updateMechanicProfile(input: {
  service_area_km?: number;
  supported_vehicle_kinds?: VehicleKind[];
  repair_categories?: RepairCategory[];
  display_name?: string;
}): Promise<MechanicProfile> {
  return api<MechanicProfile>("/api/v1/mechanics/me", {
    method: "PATCH",
    body: input,
  });
}

/** Nearby open requests feed (needs mechanic location). */
export async function nearbyRequests(
  latitude: number,
  longitude: number,
): Promise<MechanicFeedItem[]> {
  return api<MechanicFeedItem[]>(
    `/api/v1/mechanics/requests?latitude=${latitude}&longitude=${longitude}`,
  );
}

/** Live location ping (optionally tied to a job). */
export async function pushLocation(
  latitude: number,
  longitude: number,
  accuracyM?: number,
  jobId?: string,
): Promise<void> {
  await api("/api/v1/mechanics/location", {
    method: "POST",
    body: {
      latitude,
      longitude,
      accuracy_m: accuracyM ?? null,
      job_id: jobId ?? null,
    },
  });
}

export interface MechanicPayment {
  payment_id: string;
  job_id: string;
  amount_minor: number;
  method: string;
  status: string;
  receipt_number: string | null;
  created_at: string;
  confirmed_at: string | null;
  job_status: string;
  customer_email: string | null;
  customer_name: string | null;
}

/** Incoming payments for jobs the mechanic completed. */
export async function listMyEarnings(): Promise<MechanicPayment[]> {
  return api<MechanicPayment[]>("/api/v1/mechanics/payments");
}
