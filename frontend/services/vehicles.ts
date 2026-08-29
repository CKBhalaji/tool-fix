import { api } from "./api";
import type { Vehicle, VehicleKind } from "@/types";

export async function listVehicles(): Promise<Vehicle[]> {
  return api<Vehicle[]>("/api/v1/vehicles");
}

export async function createVehicle(input: {
  vehicle_kind: VehicleKind;
  make?: string;
  model?: string;
  year?: number;
  registration_number?: string;
}): Promise<Vehicle> {
  return api<Vehicle>("/api/v1/vehicles", { method: "POST", body: input });
}
