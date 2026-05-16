import { useEffect, useMemo, useState } from "react";
import { fetchAnalyticsOverview } from "../api.js";

const emptyData = {
  repo_id: "No data available",
  branches_count: 0,
  commits_count: 0,
  stars_count: 0,
  push_count: 0,
  pull_count: 0,
  last_push_at: null,
  last_pull_at: null,
  contributors_count: 0,
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

export function AnalyticsOverviewCard({ getToken, repoId }) {
  const [state, setState] = useState({
    status: "loading",
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
    fetchAnalyticsOverview(repoId, getToken)
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
  }, [getToken, repoId]);

  const data = state.data ?? { ...emptyData, repo_id: repoId ?? emptyData.repo_id };
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
          {state.status === "ready" ? "LIVE" : "NO DATA"}
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
            {compactNumber(data.push_count)} push / {compactNumber(data.pull_count)} pull
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

      {state.status === "unavailable" ? <p className="api-note">{state.error}</p> : null}
    </section>
  );
}
