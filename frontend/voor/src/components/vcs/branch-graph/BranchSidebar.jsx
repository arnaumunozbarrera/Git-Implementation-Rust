function formatRelativeTime(value) {
  if (!value) {
    return "no commits";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "recently";
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

function displayBranchName(branch) {
  return String(branch?.name || branch?.branchName || branch?.ref || branch?.slug || "branch")
    .replace(/^refs\/heads\//, "")
    .trim() || "branch";
}

function statusKey(branch, defaultBranchName) {
  const status = String(branch.status || "").toLowerCase();
  const name = displayBranchName(branch).toLowerCase();
  const defaultName = String(defaultBranchName || "").toLowerCase();
  if (branch.isDefault || branch.is_default || status === "default" || (defaultName && name === defaultName)) {
    return "default";
  }
  if (status === "outdated" || status === "stale") {
    return "outdated";
  }
  if (status === "idle") {
    return "idle";
  }

  return "active";
}

function statusLabel(status) {
  if (status === "default") {
    return "Default";
  }
  if (status === "outdated") {
    return "Stale";
  }
  if (status === "idle") {
    return "Idle";
  }

  return "Active";
}

function branchAccent(branch, status) {
  if (branch.accent) {
    return branch.accent;
  }
  if (status === "outdated") {
    return "#ffba42";
  }
  if (status === "default") {
    return "#c0c7d4";
  }

  return "#a2c9ff";
}

export function BranchSidebar({ branches, defaultBranchName, hoveredBranchName, loading, onHover }) {
  const items = Array.isArray(branches) ? branches : [];

  return (
    <aside className="vcs-branch-sidebar">
      <header className="vcs-panel-header">
        <div>
          <h2>Active Branches</h2>
        </div>
        <span className="vcs-count-pill">Total: {items.length}</span>
      </header>

      <div className="vcs-branch-list" aria-label="Repository branches">
        {items.length > 0 ? (
          items.map((branch) => {
            const latestAt = branch.latestCommit?.created_at || branch.latest_commit?.created_at || branch.updated_at || branch.created_at;
            const status = statusKey(branch, defaultBranchName);
            const ahead = branch.divergence?.ahead ?? branch.ahead ?? 0;
            const behind = branch.divergence?.behind ?? branch.behind ?? 0;
            const name = displayBranchName(branch);
            const hovered = hoveredBranchName === name;
            const muted = Boolean(hoveredBranchName && !hovered);

            return (
              <article
                className={`vcs-branch-row severity-${branch.severity || "normal"} ${hovered ? "hovered" : ""} ${muted ? "muted" : ""}`}
                key={branch.id || name}
                onBlur={() => onHover?.(null)}
                onFocus={() => onHover?.(name)}
                onMouseEnter={() => onHover?.(name)}
                onMouseLeave={() => onHover?.(null)}
                style={{ "--branch-accent": branchAccent(branch, status) }}
                tabIndex="0"
              >
                <div className="vcs-branch-row-heading">
                  <span className="material-symbols-outlined" aria-hidden="true">call_split</span>
                  <strong title={name}>{name}</strong>
                  <span className={`vcs-status-badge branch-status-${status}`}>{statusLabel(status)}</span>
                </div>

                {status === "default" ? (
                  <div className="vcs-branch-metrics">
                    <span>
                      <small>BASE</small>
                      <strong>-</strong>
                    </span>
                  </div>
                ) : (
                  <div className="vcs-branch-metrics">
                    <span className="ahead">
                      <small>AHEAD</small>
                      <strong>{ahead}</strong>
                    </span>
                    <span className="behind">
                      <small>BEHIND</small>
                      <strong>{behind}</strong>
                    </span>
                  </div>
                )}

                <div className="vcs-branch-row-footer">
                  <span>{branch.latestContributor || branch.owner?.username || branch.author?.username || "Updated"}</span>
                  <time dateTime={latestAt}>Updated {formatRelativeTime(latestAt)}</time>
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
