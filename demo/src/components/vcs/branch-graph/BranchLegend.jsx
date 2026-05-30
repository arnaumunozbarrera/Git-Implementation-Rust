import { branchStatusMeta } from "../../../utils/staleBranchDetection.js";

const fallbackStatusMeta = {
  active: {
    label: "Active",
    description: "Branch is recently updated.",
  },
  idle: {
    label: "Idle",
    description: "Branch has limited recent activity.",
  },
  outdated: {
    label: "Stale",
    description: "Branch is behind or has aging changes.",
  },
  default: {
    label: "Default",
    description: "Main repository timeline.",
  },
};

const statusOrder = ["active", "idle", "outdated", "default"];

function statusMeta(status) {
  return branchStatusMeta?.[status] || fallbackStatusMeta[status] || fallbackStatusMeta.active;
}

function branchMeta(branch) {
  if (branch.isDefault) {
    return "default timeline";
  }

  return `${branch.ahead ?? 0} ahead / ${branch.behind ?? 0} behind`;
}

export function BranchLegend({ branches, hoveredBranchName, onHover }) {
  const items = Array.isArray(branches) ? branches : [];

  if (items.length > 0) {
    return (
      <section className="branch-divergence-legend" aria-label="Branch colors and status">
        <header className="branch-divergence-legend-header">
          <span>Branch Legend</span>
          <span>Hover a branch, label, row, or legend item to isolate it</span>
        </header>
        <div className="branch-divergence-legend-list">
          {items.map((branch) => {
            const isHovered = hoveredBranchName === branch.branchName;
            const isMuted = Boolean(hoveredBranchName && !isHovered);
            const meta = statusMeta(branch.status);

            return (
              <button
                aria-label={`${branch.branchName}: ${branch.statusLabel || meta.label}, ${branchMeta(branch)}`}
                className={`branch-divergence-legend-item ${isHovered ? "active" : ""} ${isMuted ? "muted" : ""}`}
                key={branch.id || branch.branchName}
                onBlur={() => onHover?.(null)}
                onFocus={() => onHover?.(branch.branchName)}
                onMouseEnter={() => onHover?.(branch.branchName)}
                onMouseLeave={() => onHover?.(null)}
                style={{ "--branch-color": branch.color || "#58a6ff" }}
                type="button"
              >
                <span className="branch-divergence-legend-swatch" aria-hidden="true" />
                <span className="branch-divergence-legend-main">
                  <strong className="branch-divergence-legend-name" title={branch.branchName}>{branch.branchName}</strong>
                  <span className="branch-divergence-legend-meta">{branchMeta(branch)}</span>
                </span>
                <span className={`branch-divergence-status status-${branch.status || "active"}`}>{branch.statusLabel || meta.label}</span>
              </button>
            );
          })}
        </div>
      </section>
    );
  }

  return (
    <div className="branch-status-legend" aria-label="Branch status definitions">
      {statusOrder.map((status) => {
        const meta = statusMeta(status);
        return (
          <div className="branch-legend-item" key={status}>
            <span className={`branch-status-dot branch-status-${status}`} aria-hidden="true" />
            <div>
              <strong>{meta.label}</strong>
              <span>{meta.description}</span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
