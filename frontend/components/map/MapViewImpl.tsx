"use client";

// Leaflet/OpenStreetMap implementation of the MapView abstraction.
// Application components must import ./MapView, never this file directly.

import { useEffect } from "react";
import { MapContainer, Marker, TileLayer, useMap, useMapEvents } from "react-leaflet";
import L from "leaflet";
import type { MapViewProps } from "./MapView";

// Default OSM marker icons break under bundlers; build small pins instead.
function pinIcon(kind: "customer" | "mechanic" | "other") {
  const color = kind === "customer" ? "#2563eb" : kind === "mechanic" ? "#16a34a" : "#64748b";
  return L.divIcon({
    className: "",
    html: `<span style="display:block;width:16px;height:16px;border-radius:9999px;background:${color};border:3px solid white;box-shadow:0 1px 4px rgba(0,0,0,.4)"></span>`,
    iconSize: [16, 16],
    iconAnchor: [8, 8],
  });
}

function Recenter({ latitude, longitude }: { latitude: number; longitude: number }) {
  const map = useMap();
  useEffect(() => {
    map.setView([latitude, longitude], map.getZoom(), { animate: true });
  }, [latitude, longitude, map]);
  return null;
}

function ClickHandler({ onPick }: { onPick: (lat: number, lng: number) => void }) {
  useMapEvents({
    click(event) {
      onPick(event.latlng.lat, event.latlng.lng);
    },
  });
  return null;
}

export default function MapViewImpl({
  center,
  zoom = 14,
  markers = [],
  onPick,
}: MapViewProps) {
  return (
    <MapContainer
      center={[center.latitude, center.longitude]}
      zoom={zoom}
      scrollWheelZoom
      className="h-full w-full"
    >
      <TileLayer
        attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
      />
      <Recenter latitude={center.latitude} longitude={center.longitude} />
      {onPick ? <ClickHandler onPick={onPick} /> : null}
      {markers.map((marker) => (
        <Marker
          key={marker.id}
          position={[marker.latitude, marker.longitude]}
          icon={pinIcon(marker.kind ?? "other")}
          title={marker.label}
        />
      ))}
    </MapContainer>
  );
}
