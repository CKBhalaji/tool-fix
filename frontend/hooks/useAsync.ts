"use client";

import { useCallback, useEffect, useState } from "react";

interface AsyncState<T> {
  data: T | null;
  error: string | null;
  loading: boolean;
  /** Re-runs the loader (bumps an internal tick). */
  reload: () => void;
}

/**
 * Small data-loading hook so pages stay thin: pass an async loader and the
 * deps it depends on; setState happens only in promise continuations.
 */
export function useAsync<T>(
  loader: () => Promise<T>,
  deps: unknown[],
): AsyncState<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [tick, setTick] = useState(0);
  // loading starts true and flips false after the first result; reloads
  // update data in place (no spinner flicker).

  const reload = useCallback(() => setTick((t) => t + 1), []);

  useEffect(() => {
    let ignore = false;
    loader()
      .then((result) => {
        if (!ignore) {
          setData(result);
          setError(null);
        }
      })
      .catch((err: unknown) => {
        if (!ignore) {
          setError(err instanceof Error ? err.message : "Something went wrong");
        }
      })
      .finally(() => {
        if (!ignore) setLoading(false);
      });
    return () => {
      ignore = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, tick]);

  return { data, error, loading, reload };
}
