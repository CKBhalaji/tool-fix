"use client";

import { MapView } from "@/components/map/MapView";

export interface TrackingPoint {
  latitude: number;
  longitude: number;
}

/** Tracking map: breakdown location plus the live mechanic marker. */
export function TrackingMap({
  breakdown,
  mechanicPosition,
}: {
  breakdown: TrackingPoint | null;
  mechanicPosition: TrackingPoint | null;
}) {
  const markers = [];
  if (breakdown) {
    markers.push({
      id: "breakdown",
      latitude: breakdown.latitude,
      longitude: breakdown.longitude,
      kind: "customer" as const,
      label: "You",
    });
  }
  if (mechanicPosition) {
    markers.push({
      id: "mechanic",
      latitude: mechanicPosition.latitude,
      longitude: mechanicPosition.longitude,
      kind: "mechanic" as const,
      label: "Mechanic",
    });
  }

  return (
    <MapView
      center={breakdown ?? { latitude: 12.9716, longitude: 77.5946 }}
      markers={markers}
      className="h-72 w-full overflow-hidden rounded-xl border border-border"
    />
  );
}
