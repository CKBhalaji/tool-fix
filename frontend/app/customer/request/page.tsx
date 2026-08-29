"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/useAuth";
import { useGeolocation } from "@/hooks/useGeolocation";
import { Header } from "@/components/layout/Header";
import { MapView } from "@/components/map/MapView";
import { Card, PrimaryButton } from "@/components/ui";
import { apiUpload } from "@/services/api";
import { createBreakdown } from "@/services/breakdowns";
import { createVehicle, listVehicles } from "@/services/vehicles";
import type { Vehicle, VehicleKind } from "@/types";

const VEHICLE_KINDS: VehicleKind[] = ["motorcycle", "scooter", "car", "van", "truck", "other"];

/**
 * Emergency-oriented request flow: detect location -> pick vehicle ->
 * describe problem -> attach photo -> submit. The AI analysis + mechanic
 * matching pipeline then runs on the server.
 */
export default function RequestAssistancePage() {
  const { me, loading } = useAuth();
  const router = useRouter();
  const { position, locateOnce } = useGeolocation();
  const [picked, setPicked] = useState<{ latitude: number; longitude: number } | null>(null);
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);
  const [vehicleId, setVehicleId] = useState<string>("");
  const [newKind, setNewKind] = useState<VehicleKind>("motorcycle");
  const [newMake, setNewMake] = useState("");
  const [newModel, setNewModel] = useState("");
  const [description, setDescription] = useState("");
  const [symptoms, setSymptoms] = useState("");
  const [photo, setPhoto] = useState<File | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!loading && (!me || me.user.role !== "customer")) {
      router.replace("/");
    }
  }, [me, loading, router]);

  useEffect(() => {
    if (me?.user.role !== "customer") return;
    void listVehicles().then(setVehicles).catch(() => setVehicles([]));
  }, [me]);

  async function submit() {
    setError(null);
    setSubmitting(true);
    try {
      let id = vehicleId;
      if (!id) {
        const vehicle = await createVehicle({
          vehicle_kind: newKind,
          make: newMake || undefined,
          model: newModel || undefined,
        });
        id = vehicle.id;
      }
      if (!pin) throw new Error("Location is required");
      if (description.trim().length < 10) throw new Error("Please describe the problem (at least 10 characters)");

      const job = await createBreakdown({
        vehicle_id: id,
        latitude: pin.latitude,
        longitude: pin.longitude,
        problem_description: description,
        vehicle_symptoms: symptoms
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean),
      });

      if (photo) {
        const form = new FormData();
        form.append("photo", photo);
        await apiUpload(`/api/v1/breakdowns/${job.breakdown_id}/media`, form);
      }

      router.push(`/customer/tracking?job_id=${job.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Something went wrong");
    } finally {
      setSubmitting(false);
    }
  }

  // The map pin defaults to the detected position until the user picks one.
  const pin = picked ?? (position ? { latitude: position.latitude, longitude: position.longitude } : null);
  const marker = pin
    ? [{ id: "me", latitude: pin.latitude, longitude: pin.longitude, kind: "customer" as const, label: "You" }]
    : [];

  return (
    <>
      <Header title="Request assistance" />
      <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">
        <div className="grid gap-5">
          <Card>
            <div className="mb-3 flex items-center justify-between">
              <h2 className="font-semibold">1. Where are you?</h2>
              <button
                type="button"
                onClick={locateOnce}
                className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-50"
              >
                Detect my location
              </button>
            </div>
            <MapView
              center={pin ?? { latitude: 12.9716, longitude: 77.5946 }}
              markers={marker}
              onPick={(latitude, longitude) => setPicked({ latitude, longitude })}
              className="h-64 w-full overflow-hidden rounded-xl border border-slate-200"
            />
            {pin ? (
              <p className="mt-2 text-xs text-slate-500">
                {pin.latitude.toFixed(5)}, {pin.longitude.toFixed(5)}
                {position?.accuracy ? ` (±${Math.round(position.accuracy)} m)` : ""}
              </p>
            ) : (
              <p className="mt-2 text-xs text-slate-500">Tap the map to pin your exact spot.</p>
            )}
          </Card>

          <Card>
            <h2 className="mb-3 font-semibold">2. Your vehicle</h2>
            {vehicles.length > 0 ? (
              <select
                value={vehicleId}
                onChange={(event) => setVehicleId(event.target.value)}
                className="w-full rounded-xl border border-slate-300 px-3 py-2.5 text-sm"
              >
                <option value="">— Add a new vehicle —</option>
                {vehicles.map((vehicle) => (
                  <option key={vehicle.id} value={vehicle.id}>
                    {vehicle.vehicle_kind} · {[vehicle.make, vehicle.model, vehicle.year].filter(Boolean).join(" ")}
                  </option>
                ))}
              </select>
            ) : null}
            {(vehicles.length === 0 || !vehicleId) && (
              <div className="mt-3 grid gap-3 sm:grid-cols-3">
                <select
                  value={newKind}
                  onChange={(event) => setNewKind(event.target.value as VehicleKind)}
                  className="rounded-xl border border-slate-300 px-3 py-2.5 text-sm"
                >
                  {VEHICLE_KINDS.map((kind) => (
                    <option key={kind} value={kind}>{kind}</option>
                  ))}
                </select>
                <input
                  value={newMake}
                  onChange={(event) => setNewMake(event.target.value)}
                  placeholder="Make (e.g. Honda)"
                  className="rounded-xl border border-slate-300 px-3 py-2.5 text-sm"
                />
                <input
                  value={newModel}
                  onChange={(event) => setNewModel(event.target.value)}
                  placeholder="Model (e.g. Activa)"
                  className="rounded-xl border border-slate-300 px-3 py-2.5 text-sm"
                />
              </div>
            )}
          </Card>

          <Card>
            <h2 className="mb-3 font-semibold">3. What happened?</h2>
            <textarea
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              rows={3}
              placeholder="e.g. Bike suddenly stopped while riding. Engine turns but does not start."
              className="w-full rounded-xl border border-slate-300 px-3 py-2.5 text-sm"
            />
            <input
              value={symptoms}
              onChange={(event) => setSymptoms(event.target.value)}
              placeholder="Symptoms, comma separated (e.g. clicking sound, warning light)"
              className="mt-3 w-full rounded-xl border border-slate-300 px-3 py-2.5 text-sm"
            />
            <input
              type="file"
              accept="image/*"
              capture="environment"
              onChange={(event) => setPhoto(event.target.files?.[0] ?? null)}
              className="mt-3 block w-full text-sm text-slate-600"
            />
            {photo ? <p className="mt-1 text-xs text-slate-500">Attached: {photo.name}</p> : null}
          </Card>

          {error ? <p className="text-sm text-red-600">{error}</p> : null}
          <PrimaryButton onClick={submit} disabled={submitting} className="w-full !py-3.5 text-base">
            {submitting ? "Sending…" : "Request help now"}
          </PrimaryButton>
        </div>
      </main>
    </>
  );
}
