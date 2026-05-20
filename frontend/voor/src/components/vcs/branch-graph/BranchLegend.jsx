import { branchStatusMeta } from "../../../utils/staleBranchDetection.js";

const statusOrder = ["active", "idle", "outdated", "default"];

export function BranchLegend() {
  return (
    <div className="branch-status-legend" aria-label="Branch status definitions">
      {statusOrder.map((status) => {
        const meta = branchStatusMeta[status];
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
