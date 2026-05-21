import { useState } from "react";
import { useBranchMetrics } from "../../../hooks/useBranchMetrics.js";
import { useBranchTopology } from "../../../hooks/useBranchTopology.js";
import { useRepositoryAnalytics } from "../../../hooks/useRepositoryAnalytics.js";
import { BranchMetrics } from "./BranchMetrics.jsx";
import { BranchLabel } from "./BranchLabel.jsx";
import { BranchNode } from "./BranchNode.jsx";
import { BranchPath } from "./BranchPath.jsx";
import { BranchSidebar } from "./BranchSidebar.jsx";
import { BranchTooltip } from "./BranchTooltip.jsx";

function gradientId(value) {
  return String(value).replace(/[^a-zA-Z0-9_-]/g, "-");
}

export function BranchGraph({ getToken, repository }) {
  const [hoveredCommit, setHoveredCommit] = useState(null);
  const analyticsState = useRepositoryAnalytics({
    getToken,
    repoId: repository?.id,
    repository,
  });
  const data = analyticsState.data || {};
  const branches = data.branches || [];
  const topology = useBranchTopology({
    branches,
    graphsByBranch: data.graphsByBranch || {},
    hoveredBranchName: "",
    repository,
    selectedBranchName: "",
  });
  const metrics = useBranchMetrics({ analytics: data.analytics, branches });
  const loading = analyticsState.status === "loading";

  return (
    <section className="workspace-section">
      <div className="landing-heading">
        <p className="label-caps">Repository Intelligence</p>
        <h1>Branches</h1>
        <p>Branch health, divergence, commit topology, and repository audit signals for the active repository.</p>
      </div>

      <div className="vcs-observability-layout">
        <BranchSidebar
          branches={branches}
          loading={loading}
        />

        <section className="vcs-graph-panel">
          <header className="vcs-panel-header">
            <div>
              <h2>Branch Divergence Visualization</h2>
            </div>
            <span className="vcs-count-pill">{branches.length} branches</span>
          </header>

          <div className="branch-graph-stage">
            {branches.length > 0 ? (
              <>
                <svg className="branch-topology-svg" viewBox={`0 0 ${topology.width} ${topology.height}`} role="img" aria-label="Commit topology graph" xmlns="http://www.w3.org/2000/svg">
                  <defs>
                    <pattern id="topology-grid" width="52" height="52" patternUnits="userSpaceOnUse">
                      <path d="M 52 0 L 0 0 0 52" fill="none" stroke="rgba(139,145,157,0.075)" strokeWidth="1" />
                      <path d="M 0 26 L 52 26 M 26 0 L 26 52" fill="none" stroke="rgba(139,145,157,0.035)" strokeWidth="1" />
                    </pattern>
                    {topology.paths.map((path) => (
                      <linearGradient id={`gradient-${gradientId(path.id)}`} key={path.id} x1="0%" x2="100%" y1="0%" y2="0%">
                        <stop offset="0%" stopColor="rgba(139,145,157,0.44)" />
                        <stop offset="58%" stopColor={path.color} stopOpacity="0.78" />
                        <stop offset="100%" stopColor={path.color} />
                      </linearGradient>
                    ))}
                  </defs>
                  <rect width={topology.width} height={topology.height} fill="url(#topology-grid)" />
                  <g className="graph-heat-layer">
                    {topology.heatZones.map((zone) => (
                      <rect
                        height={zone.height}
                        key={zone.id}
                        rx="20"
                        style={{ fill: zone.color, opacity: zone.opacity }}
                        width={zone.width}
                        x={zone.x}
                        y={zone.y}
                      />
                    ))}
                  </g>
                  <g className="graph-baseline">
                    <line x1="34" x2={topology.width - 34} y1={topology.centerY} y2={topology.centerY} />
                  </g>
                  <g>
                    {topology.paths.map((path) => (
                      <BranchPath key={path.id} path={path} />
                    ))}
                  </g>
                  <g>
                    {topology.nodes.map((node) => (
                      <BranchNode
                        active
                        hovered={hoveredCommit?.hash === node.hash}
                        key={node.hash}
                        node={node}
                        onHover={setHoveredCommit}
                      />
                    ))}
                  </g>
                  <g className="branch-label-layer">
                    {topology.labels.map((label) => (
                      <BranchLabel key={label.id} label={label} />
                    ))}
                  </g>
                </svg>
                <BranchTooltip node={hoveredCommit} />
              </>
            ) : (
              <div className="vcs-graph-empty">
                <span className="material-symbols-outlined" aria-hidden="true">account_tree</span>
                <p>{loading ? "Building repository topology..." : analyticsState.error || "No topology data available"}</p>
              </div>
            )}
          </div>

          <BranchMetrics metrics={metrics} />
        </section>
      </div>
    </section>
  );
}
