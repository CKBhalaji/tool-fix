"use client";

import { useCallback, useEffect, useRef, useState } from "react";

export interface GeoPosition {
  latitude: number;
  longitude: number;
  accuracy: number | null;
}

/**
 * Browser geolocation wrapper: one-shot position plus an optional watch
 * used by the mechanic's online/location-sharing mode.
 */
export function useGeolocation(watch = false) {
  const [position, setPosition] = useState<GeoPosition | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(watch);
  const watchId = useRef<number | null>(null);

  const update = useCallback((pos: GeolocationPosition) => {
    setPosition({
      latitude: pos.coords.latitude,
      longitude: pos.coords.longitude,
      accuracy: pos.coords.accuracy ?? null,
    });
    setError(null);
    setLoading(false);
  }, []);

  const fail = useCallback((err: GeolocationPositionError) => {
    setError(err.message || "Could not determine location");
    setLoading(false);
  }, []);

  const locateOnce = useCallback(() => {
    if (!("geolocation" in navigator)) {
      setError("Geolocation is not available in this browser");
      return;
    }
    setLoading(true);
    navigator.geolocation.getCurrentPosition(update, fail, {
      enableHighAccuracy: true,
      timeout: 10_000,
    });
  }, [update, fail]);

  useEffect(() => {
    if (!watch || !("geolocation" in navigator)) return;
    watchId.current = navigator.geolocation.watchPosition(update, fail, {
      enableHighAccuracy: true,
      timeout: 15_000,
      maximumAge: 5_000,
    });
    return () => {
      if (watchId.current !== null) {
        navigator.geolocation.clearWatch(watchId.current);
      }
    };
  }, [watch, update, fail]);

  return { position, error, loading, locateOnce };
}
