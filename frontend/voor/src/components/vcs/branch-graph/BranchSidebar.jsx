import { BranchLegend } from "./BranchLegend.jsx";

function formatRelativeTime(value) {
  if (!value) {
    return "no commits";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "";
  }

  const seconds = Math.round((date.getTime() - Date.now()) / 1000);
  const divisions = [
    { amount: 60, unit: "second" },
    { amount: 60, unit: "minute" },
    { amount: 24, unit: "hour" },
    { amount: 7, unit: "day" },
    { amount: 4.345, unit: "week" },
    { amount: 12, unit: "month" },
    { amount: Number.POSITIVE_INFINITY, unit: "year" },
  ];

  let duration = seconds;
  for (const division of divisions) {
    if (Math.abs(duration) < division.amount) {
      return new Intl.RelativeTimeFormat("en", { numeric: "auto" }).format(Math.round(duration), division.unit);
    }
    duration /= division.amount;
  }

  return "recently";
}

function formatScore(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) {
    return "";
  }

  return `${Math.round(number)}`;
}

export function BranchSidebar({ branches, loading }) {
  return (
    <aside className="vcs-branch-sidebar">
      <header className="vcs-panel-header">
        <div>
          <h2>Branch Status Overview</h2>
        </div>
        <span className="vcs-count-pill">Total: {branches.length}</span>
      </header>

      <BranchLegend />

      <div className="vcs-branch-list" aria-label="Repository branches">
        {branches.length > 0 ? (
          branches.map((branch) => {
            const latestAt = branch.latestCommit?.created_at || branch.created_at;

            return (
              <article
                className={`vcs-branch-row severity-${branch.severity}`}
                key={branch.id || branch.name}
                style={{ "--branch-accent": branch.accent }}
              >
                <div className="vcs-branch-row-heading">
                  <span className="material-symbols-outlined" aria-hidden="true">account_tree</span>
                  <strong>{branch.name}</strong>
                  <span className={`vcs-status-badge branch-status-${branch.status}`}>{branch.status}</span>
                </div>
                <div className="vcs-branch-metrics">
                  <span>
                    <small>AHEAD</small>
                    <strong>{branch.divergence?.ahead ?? 0}</strong>
                  </span>
                  <span>
                    <small>BEHIND</small>
                    <strong>{branch.divergence?.behind ?? 0}</strong>
                  </span>
                  <span>
                    <small>DIST</small>
                    <strong>{branch.divergence?.distance ?? 0}</strong>
                  </span>
                  <span>
                    <small>HEALTH</small>
                    <strong>{formatScore(branch.health_score)}</strong>
                  </span>
                  <span>
                    <small>FRESH</small>
                    <strong>{formatScore(branch.freshness_score)}</strong>
                  </span>
                  <span>
                    <small>STALE</small>
                    <strong>{Number(branch.stale_days) || 0}d</strong>
                  </span>
                </div>
                <div className="vcs-branch-row-footer">
                  <span>{branch.latestContributor}</span>
                  <time dateTime={latestAt}>{formatRelativeTime(latestAt)}</time>
                </div>
                <div className="vcs-activity-track" aria-hidden="true">
                  <span style={{ width: `${Math.min(100, 18 + (branch.commitCount || 0) * 8)}%` }} />
                </div>
              </article>
            );
          })
        ) : (
          <p className="vcs-empty-state">{loading ? "Loading branch intelligence..." : "No branch data available"}</p>
        )}
      </div>
    </aside>
  );
}
