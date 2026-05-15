import { useEffect, useMemo, useState } from "react";
import { fetchAnalyticsOverview } from "../api.js";

const fallbackData = {
  repo_id: import.meta.env.VITE_REPO_ID ?? "main-repo-v2",
  branches_count: 12,
  commits_count: 14820,
  stars_count: 284,
  push_count: 732,
  pull_count: 691,
  last_push_at: "2026-05-15T14:48:00Z",
  last_pull_at: "2026-05-15T14:34:00Z",
  contributors_count: 18,
};

function compactNumber(value) {
  return new Intl.NumberFormat("en", { notation: "compact" }).format(value);
}

function formatActivityTime(value) {
  if (!value) {
    return "no activity";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

export function AnalyticsOverviewCard() {
  const repoId = import.meta.env.VITE_REPO_ID ?? "main-repo-v2";
  const [state, setState] = useState({
    status: "loading",
    data: null,
    error: null,
  });

  useEffect(() => {
    let active = true;

    fetchAnalyticsOverview(repoId)
      .then((data) => {
        if (active) {
          setState({ status: "ready", data, error: null });
        }
      })
      .catch((error) => {
        if (active) {
          setState({ status: "fallback", data: fallbackData, error: error.message });
        }
      });

    return () => {
      active = false;
    };
  }, [repoId]);

  const data = state.data ?? fallbackData;
  const syncRatio = useMemo(() => {
    const total = data.push_count + data.pull_count;
    if (total === 0) {
      return 0;
    }

    return Math.round((data.push_count / total) * 100);
  }, [data.pull_count, data.push_count]);

  return (
    <section className="analytics-card" aria-label="Analytics overview">
      <header className="card-header">
        <div>
          <p className="label-caps">Analytics Overview</p>
          <h2>{data.repo_id}</h2>
        </div>
        <span className={`status-badge ${state.status === "ready" ? "status-live" : "status-sample"}`}>
          {state.status === "ready" ? "LIVE" : "SAMPLE"}
        </span>
      </header>

      <div className="metric-grid">
        <div className="metric">
          <span className="metric-label">Commits</span>
          <strong>{compactNumber(data.commits_count)}</strong>
        </div>
        <div className="metric">
          <span className="metric-label">Branches</span>
          <strong>{compactNumber(data.branches_count)}</strong>
        </div>
        <div className="metric">
          <span className="metric-label">Contributors</span>
          <strong>{compactNumber(data.contributors_count)}</strong>
        </div>
        <div className="metric">
          <span className="metric-label">Stars</span>
          <strong>{compactNumber(data.stars_count)}</strong>
        </div>
      </div>

      <div className="sync-row">
        <div>
          <span className="metric-label">Push / Pull Ratio</span>
          <p className="mono-line">
            {compactNumber(data.push_count)} push · {compactNumber(data.pull_count)} pull
          </p>
        </div>
        <strong className="sync-value">{syncRatio}%</strong>
      </div>
      <div className="ratio-track" aria-hidden="true">
        <span style={{ width: `${syncRatio}%` }} />
      </div>

      <footer className="activity-footer">
        <span>last push {formatActivityTime(data.last_push_at)}</span>
        <span>last pull {formatActivityTime(data.last_pull_at)}</span>
      </footer>

      {state.status === "fallback" ? <p className="api-note">{state.error}</p> : null}
    </section>
  );
}
