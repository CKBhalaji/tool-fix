import { api } from "./api";
import type { Offer } from "@/types";

/** Customer: compare incoming offers for a job. */
export async function listOffers(jobId: string): Promise<Offer[]> {
  return api<Offer[]>(`/api/v1/jobs/${jobId}/offers`);
}

/** Mechanic: submit a competitive bid. */
export async function createOffer(
  jobId: string,
  input: {
    quoted_price_minor: number;
    estimated_arrival_minutes: number;
    message?: string;
  },
): Promise<Offer> {
  return api<Offer>(`/api/v1/jobs/${jobId}/offers`, {
    method: "POST",
    body: input,
  });
}

/** Customer accepts one offer; competing offers expire atomically. */
export async function selectOffer(offerId: string): Promise<unknown> {
  return api(`/api/v1/offers/${offerId}/select`, { method: "POST" });
}
