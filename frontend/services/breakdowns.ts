import { api, apiUpload } from "./api";
import type {
  BreakdownDetail,
  Diagnosis,
  Job,
  PriceEstimate,
} from "@/types";

export interface BreakdownInput {
  vehicle_id: string;
  latitude: number;
  longitude: number;
  address?: string;
  problem_description: string;
  vehicle_symptoms: string[];
}

export async function createBreakdown(input: BreakdownInput): Promise<Job> {
  return api<Job>("/api/v1/breakdowns", { method: "POST", body: input });
}

export async function getBreakdown(id: string): Promise<BreakdownDetail> {
  return api<BreakdownDetail>(`/api/v1/breakdowns/${id}`);
}

export async function cancelBreakdown(id: string): Promise<Job> {
  return api<Job>(`/api/v1/breakdowns/${id}/cancel`, { method: "POST" });
}

export async function uploadMedia(
  breakdownId: string,
  kind: "photo" | "video",
  file: File,
): Promise<void> {
  const form = new FormData();
  form.append(kind, file);
  await apiUpload(`/api/v1/breakdowns/${breakdownId}/media`, form);
}

export async function getDiagnosis(breakdownId: string): Promise<{
  breakdown_id: string;
  diagnosis: Diagnosis;
  price_estimate: PriceEstimate;
}> {
  return api(`/api/v1/breakdowns/${breakdownId}/diagnosis`);
}
