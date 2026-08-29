"use client";

import dynamic from "next/dynamic";
import "leaflet/dist/leaflet.css";

/**
 * MapView — vendor-abstracted map component.
 *
 * Application code depends on this interface only (center, markers,
 * pick mode, live position). Leaflet/OpenStreetMap is the current
 * implementation, hidden behind the dynamic import below so swapping to
 * Google Maps or Mappls later does not touch callers.
 */
export interface MapMarker {
  id: string;
  latitude: number;
  longitude: number;
  label?: string;
  kind?: "customer" | "mechanic" | "other";
}

export interface MapViewProps {
  center: { latitude: number; longitude: number };
  zoom?: number;
  markers?: MapMarker[];
  /** Picker mode: clicking the map reports coordinates via onPick. */
  onPick?: (latitude: number, longitude: number) => void;
  /** Live position updates without page refresh (mechanic tracking). */
  className?: string;
}

// Leaflet touches `window`, so the implementation is client-only.
const MapViewImpl = dynamic(() => import("./MapViewImpl"), {
  ssr: false,
  loading: () => (
    <div className="flex h-full w-full items-center justify-center rounded-xl bg-slate-100 text-sm text-slate-500">
      Loading map…
    </div>
  ),
});

export function MapView(props: MapViewProps) {
  return (
    <div className={props.className ?? "h-72 w-full overflow-hidden rounded-xl border border-slate-200"}>
      <MapViewImpl {...props} />
    </div>
  );
}
