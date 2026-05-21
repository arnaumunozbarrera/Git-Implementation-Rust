import { useEffect, useState } from "react";
import { fetchRepositoryAnalytics } from "../services/branchAnalyticsService.js";

export function useRepositoryAnalytics({ getToken, repoId, repository }) {
  const [state, setState] = useState({
    status: "idle",
    data: null,
    error: null,
  });

  useEffect(() => {
    let active = true;

    if (!repoId) {
      setState({ status: "empty", data: null, error: null });
      return () => {
        active = false;
      };
    }

    setState({ status: "loading", data: null, error: null });
    fetchRepositoryAnalytics({ repoId, getToken, repository })
      .then((data) => {
        if (active) {
          setState({ status: "ready", data, error: null });
        }
      })
      .catch((error) => {
        if (active) {
          setState({ status: "unavailable", data: null, error: error.message });
        }
      });

    return () => {
      active = false;
    };
  }, [getToken, repoId, repository]);

  return state;
}
