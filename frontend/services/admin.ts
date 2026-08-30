// Admin console API. Login uses the backend's static admin credentials and
// relies on the same HttpOnly cookie session as everyone else.

import { api } from "./api";
import type { JobStatus, UserStatus } from "@/types";

export interface AdminOverview {
  users_total: number;
  customers: number;
  mechanics: number;
  admins: number;
  jobs_total: number;
  jobs_active: number;
  jobs_completed: number;
  offers_total: number;
  mechanics_verified: number;
  payments_collected_minor: number;
  payments_count: number;
  ai_runs_total: number;
}

export interface AdminPayment {
  payment_id: string;
  job_id: string;
  amount_minor: number;
  method: string;
  status: string;
  provider: string;
  receipt_number: string | null;
  created_at: string;
  confirmed_at: string | null;
  customer_email: string | null;
  customer_name: string | null;
  job_status: string;
}

export interface AdminAgentRun {
  run_id: string;
  breakdown_id: string;
  job_id: string | null;
  kind: string;
  provider: string;
  model: string | null;
  status: string;
  error_message: string | null;
  latency_ms: number | null;
  created_at: string;
  problem_description: string;
}

export interface AdminUser {
  id: string;
  email: string | null;
  display_name: string | null;
  phone: string | null;
  role: string;
  status: string;
  onboarding_status: string;
  created_at: string;
}

export interface AdminMechanic {
  mechanic_id: string;
  display_name: string | null;
  phone: string | null;
  city: string | null;
  service_area_km: number;
  availability_status: string;
  rating_average: number | null;
  completed_jobs: number;
  is_verified: boolean;
  current_latitude: number | null;
  current_longitude: number | null;
  user: {
    email: string | null;
    display_name: string | null;
  };
}

export interface AdminJob {
  job_id: string;
  status: JobStatus;
  final_amount_minor: number | null;
  created_at: string;
  breakdown_id: string;
  problem_description: string;
  latitude: number;
  longitude: number;
  customer_email: string | null;
  customer_name: string | null;
  mechanic_id: string | null;
  mechanic_name: string | null;
}

export async function adminLogin(
  email: string,
  password: string,
): Promise<void> {
  await api("/api/v1/admin/login", {
    method: "POST",
    body: { email, password },
    skipRefresh: true,
  });
}

export async function overview(): Promise<AdminOverview> {
  return api<AdminOverview>("/api/v1/admin/overview");
}

export async function listUsers(role?: string): Promise<AdminUser[]> {
  const query = role ? `?role=${role}` : "";
  return api<AdminUser[]>(`/api/v1/admin/users${query}`);
}

export async function setUserStatus(
  userId: string,
  status: UserStatus,
): Promise<void> {
  await api(`/api/v1/admin/users/${userId}/status`, {
    method: "POST",
    body: { status },
  });
}

export async function listMechanics(): Promise<AdminMechanic[]> {
  return api<AdminMechanic[]>("/api/v1/admin/mechanics");
}

export async function setVerified(
  mechanicId: string,
  verified: boolean,
): Promise<void> {
  await api(`/api/v1/admin/mechanics/${mechanicId}/verify`, {
    method: "POST",
    body: { verified },
  });
}

export async function listJobs(): Promise<AdminJob[]> {
  return api<AdminJob[]>("/api/v1/admin/jobs");
}

export async function listPayments(): Promise<AdminPayment[]> {
  return api<AdminPayment[]>("/api/v1/admin/payments");
}

export async function listAgentRuns(): Promise<AdminAgentRun[]> {
  return api<AdminAgentRun[]>("/api/v1/admin/agent-runs");
}
