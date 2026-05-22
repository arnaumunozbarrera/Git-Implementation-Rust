import { useMemo, useRef, useState } from "react";
import { useBranchMetrics } from "../../../hooks/useBranchMetrics.js";
import { useRepositoryAnalytics } from "../../../hooks/useRepositoryAnalytics.js";
import { BranchMetrics } from "./BranchMetrics.jsx";
import { BranchLabel } from "./BranchLabel.jsx";
import { BranchPath } from "./BranchPath.jsx";
import { BranchSidebar } from "./BranchSidebar.jsx";
import { BranchLegend } from "./BranchLegend.jsx";

function gradientId(value) {
  return String(value).replace(/[^a-zA-Z0-9_-]/g, "-");
}

function firstNumber(...values) {
  for (const value of values) {
    if (value === null || value === undefined || value === "") {
      continue;
    }

    const normalized = typeof value === "string"
      ? Number(value.replace(/[^0-9.-]/g, ""))
      : Number(value);

    if (Number.isFinite(normalized)) {
      return normalized;
    }
  }

  return null;
}

function formatNumber(value) {
  const number = firstNumber(value);
  if (number === null) {
    return "-";
  }

  return new Intl.NumberFormat("en", { maximumFractionDigits: 0 }).format(number);
}

function formatDecimal(value, digits = 2) {
  const number = firstNumber(value);
  if (number === null) {
    return "-";
  }

  return number.toFixed(digits);
}

function formatRelativeTime(value) {
  if (!value) {
    return "recently";
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

function normalizeComplexity(value, fallback) {
  const raw = firstNumber(value, fallback);
  if (raw === null) {
    return 0;
  }

  return Math.max(0, Math.min(1, raw > 1 ? raw / 100 : raw));
}

function buildRecentDagEvents(data, branches) {
  const eventSources = [
    data.recentDagModifications,
    data.recentDagEvents,
    data.recentModifications,
    data.recentCommits,
    data.commits,
  ];

  const directEvents = eventSources.find(Array.isArray) || [];
  const branchEvents = branches
    .map((branch) => {
      const commit = branch.latestCommit || branch.latest_commit;
      if (!commit) {
        return null;
      }

      return {
        branchName: branch.name,
        created_at: commit.created_at || commit.createdAt || branch.updated_at || branch.created_at,
        hash: commit.hash || commit.sha || commit.id,
        message: commit.message || `Latest commit on ${branch.name}`,
        tone: branch.status === "outdated" || branch.status === "stale" ? "warn" : "success",
      };
    })
    .filter(Boolean);

  return [...directEvents, ...branchEvents]
    .map((event) => {
      const message = event.message || event.title || event.summary || "Repository DAG update";
      const text = `${message} ${event.action || ""}`.toLowerCase();
      const tone = event.tone || (text.includes("delete") || text.includes("remove") || text.includes("deprecated") ? "danger" : "success");

      return {
        created_at: event.created_at || event.createdAt || event.timestamp || event.date,
        hash: event.hash || event.sha || event.commit_hash || event.id || "",
        message,
        tone,
      };
    })
    .sort((a, b) => new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime())
    .slice(0, 3);
}

function shortHash(value) {
  return String(value || "").slice(0, 7) || "pending";
}

function repositorySlug(repository) {
  return String(repository?.name || repository?.slug || "branch-divergence")
    .trim()
    .replace(/[^a-zA-Z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "") || "branch-divergence";
}


const BRANCH_COLOR_PALETTE = [
  "#58a6ff",
  "#bc8cff",
  "#3fb950",
  "#ff7b72",
  "#ffa657",
  "#79c0ff",
  "#d2a8ff",
  "#7ee787",
  "#f2cc60",
  "#a5d6ff",
];

function clampNumber(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function branchName(branch) {
  return String(branch?.name || branch?.branchName || branch?.ref || branch?.slug || "branch")
    .replace(/^refs\/heads\//, "")
    .trim() || "branch";
}

function latestCommit(branch) {
  return branch?.latestCommit || branch?.latest_commit || branch?.headCommit || branch?.head_commit || null;
}

function latestCommitDate(branch) {
  const commit = latestCommit(branch);
  return commit?.created_at || commit?.createdAt || commit?.committed_at || branch?.updated_at || branch?.created_at || "";
}

function branchDivergence(branch) {
  const divergence = branch?.divergence || {};
  const ahead = firstNumber(
    divergence.ahead,
    divergence.commitsAhead,
    divergence.commits_ahead,
    branch?.ahead,
    branch?.aheadBy,
    branch?.commitsAhead,
    branch?.commits_ahead,
    branch?.ahead_count,
  ) || 0;
  const behind = firstNumber(
    divergence.behind,
    divergence.commitsBehind,
    divergence.commits_behind,
    branch?.behind,
    branch?.behindBy,
    branch?.commitsBehind,
    branch?.commits_behind,
    branch?.behind_count,
  ) || 0;

  return {
    ahead: Math.max(0, Math.round(ahead)),
    behind: Math.max(0, Math.round(behind)),
  };
}

function branchStatusKey(branch, isDefault = false) {
  const status = String(branch?.status || branch?.state || "").toLowerCase();

  if (isDefault || branch?.isDefault || branch?.is_default || status === "default") {
    return "default";
  }
  if (status === "outdated" || status === "stale" || status === "deprecated") {
    return "outdated";
  }
  if (status === "idle" || status === "inactive") {
    return "idle";
  }

  return "active";
}

function branchStatusLabel(status) {
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

function repositoryDefaultBranchName(repository) {
  return String(
    repository?.defaultBranch ||
    repository?.default_branch ||
    repository?.mainBranch ||
    repository?.main_branch ||
    repository?.primaryBranch ||
    repository?.primary_branch ||
    "",
  ).trim().toLowerCase();
}

function resolveDefaultBranchIndex(branches, repository) {
  const items = Array.isArray(branches) ? branches : [];
  const repoDefault = repositoryDefaultBranchName(repository);

  const explicitIndex = items.findIndex((branch) => {
    const status = String(branch?.status || branch?.state || "").toLowerCase();
    return Boolean(branch?.isDefault || branch?.is_default || status === "default");
  });
  if (explicitIndex >= 0) {
    return explicitIndex;
  }

  if (repoDefault) {
    const repoDefaultIndex = items.findIndex((branch) => branchName(branch).toLowerCase() === repoDefault);
    if (repoDefaultIndex >= 0) {
      return repoDefaultIndex;
    }
  }

  const mainIndex = items.findIndex((branch) => branchName(branch).toLowerCase() === "main");
  if (mainIndex >= 0) {
    return mainIndex;
  }

  const masterIndex = items.findIndex((branch) => branchName(branch).toLowerCase() === "master");
  if (masterIndex >= 0) {
    return masterIndex;
  }

  return 0;
}

function branchColor(branch, branchIndex, isDefault) {
  if (isDefault) {
    return "#c0c7d4";
  }

  const explicitColor = branch?.color || branch?.accent || branch?.branchColor || branch?.branch_color;
  if (explicitColor) {
    return explicitColor;
  }

  return BRANCH_COLOR_PALETTE[Math.abs(branchIndex) % BRANCH_COLOR_PALETTE.length];
}

function buildTimelineTicks(minUnit, maxUnit, unitToX, height) {
  const span = Math.max(1, maxUnit - minUnit);
  const step = span <= 6 ? 1 : span <= 14 ? 2 : span <= 35 ? 5 : span <= 80 ? 10 : span <= 160 ? 20 : span <= 350 ? 50 : 100;
  const values = new Set([0, minUnit, maxUnit]);
  const first = Math.ceil(minUnit / step) * step;

  for (let value = first; value <= maxUnit; value += step) {
    values.add(value);
  }

  return Array.from(values)
    .filter((value) => Number.isFinite(value) && value >= minUnit && value <= maxUnit)
    .sort((a, b) => a - b)
    .map((value) => {
      const rounded = Math.round(value);
      const label = rounded === 0
        ? "default"
        : rounded < 0
          ? `${Math.abs(rounded)} behind`
          : `${rounded} ahead`;

      return {
        id: `tick-${rounded}`,
        x: unitToX(value),
        y: height - 54,
        value: rounded,
        label,
      };
    });
}

function buildDivergenceTopology({ branches, repository, hoveredBranchName }) {
  const sourceBranches = Array.isArray(branches) ? branches.filter(Boolean) : [];

  if (!sourceBranches.length) {
    return {
      paths: [],
      labels: [],
      heatZones: [],
      axisTicks: [],
      legendItems: [],
      width: 1040,
      height: 430,
      centerY: 215,
      defaultBranchName: "main",
    };
  }

  const defaultIndex = resolveDefaultBranchIndex(sourceBranches, repository);
  const defaultBranch = sourceBranches[defaultIndex] || sourceBranches[0];
  const defaultName = branchName(defaultBranch);
  const normalizedBranches = sourceBranches.map((branch, index) => {
    const isDefault = index === defaultIndex;
    const status = branchStatusKey(branch, isDefault);
    const divergence = branchDivergence(branch);

    return {
      branch,
      id: branch?.id || branch?.uuid || branchName(branch),
      name: branchName(branch),
      sourceIndex: index,
      isDefault,
      status,
      statusLabel: branchStatusLabel(status),
      color: branchColor(branch, index, isDefault),
      ahead: isDefault ? 0 : divergence.ahead,
      behind: isDefault ? 0 : divergence.behind,
      health: firstNumber(branch?.health, branch?.score, branch?.stabilityScore),
      latestAt: latestCommitDate(branch),
      latestCommit: latestCommit(branch),
      severity: branch?.severity || (status === "outdated" ? "warning" : "normal"),
    };
  });

  const nonDefaultBranches = normalizedBranches.filter((branch) => !branch.isDefault);
  const topLaneCount = Math.ceil(nonDefaultBranches.length / 2);
  const bottomLaneCount = Math.floor(nonDefaultBranches.length / 2);
  const laneSpacing = 72;
  const topPadding = 86;
  const bottomPadding = 116;
  const neededHeight = topPadding + bottomPadding + (topLaneCount + bottomLaneCount) * laneSpacing;
  const height = Math.max(430, neededHeight);
  const centerY = nonDefaultBranches.length > 0 ? topPadding + topLaneCount * laneSpacing : Math.round(height / 2);

  const timelineUnits = nonDefaultBranches.flatMap((branch) => [
    -branch.behind,
    branch.ahead - branch.behind,
  ]);
  const minUnit = Math.min(-1, 0, ...timelineUnits);
  const maxUnit = Math.max(1, 0, ...timelineUnits);
  const unitSpan = Math.max(1, maxUnit - minUnit);
  const width = Math.max(1040, Math.min(1800, 720 + unitSpan * 48));
  const leftPadding = 96;
  const rightPadding = 130;
  const plotWidth = Math.max(320, width - leftPadding - rightPadding);
  const unitToX = (unit) => leftPadding + ((unit - minUnit) / unitSpan) * plotWidth;
  const timelineStartX = unitToX(minUnit);
  const timelineEndX = unitToX(maxUnit);
  const defaultHeadX = unitToX(0);
  const activeBranchName = hoveredBranchName || "";
  const isPathMuted = (name) => Boolean(activeBranchName && activeBranchName !== name);
  const defaultLegendBranch = normalizedBranches.find((branch) => branch.isDefault);
  const paths = [];
  const labels = [];

  paths.push({
    id: `path-${defaultName}`,
    branchName: defaultName,
    status: "default",
    statusLabel: "Default",
    severity: "normal",
    color: defaultLegendBranch?.color || "#c0c7d4",
    d: `M ${timelineStartX} ${centerY} L ${timelineEndX} ${centerY}`,
    isDefault: true,
    ahead: 0,
    behind: 0,
    active: !activeBranchName || activeBranchName === defaultName,
    muted: isPathMuted(defaultName),
  });

  labels.push({
    id: `label-${defaultName}`,
    branchName: defaultName,
    status: "default",
    statusLabel: "Default",
    color: defaultLegendBranch?.color || "#c0c7d4",
    isDefault: true,
    ahead: 0,
    behind: 0,
    x: clampNumber(defaultHeadX + 16, 16, width - 160),
    y: clampNumber(centerY - 34, 18, height - 46),
  });

  nonDefaultBranches.forEach((branch, index) => {
    const direction = index % 2 === 0 ? -1 : 1;
    const laneMagnitude = Math.floor(index / 2) + 1;
    const laneY = centerY + direction * laneMagnitude * laneSpacing;
    const forkUnit = -branch.behind;
    const headUnit = branch.ahead - branch.behind;
    const forkX = unitToX(forkUnit);
    const headX = unitToX(headUnit);
    const horizontalDistance = Math.abs(headX - forkX);
    const horizontalDirection = headX >= forkX ? 1 : -1;
    const bendX = horizontalDistance < 2
      ? forkX
      : forkX + horizontalDirection * clampNumber(horizontalDistance * 0.36, 28, 58);
    const pathD = horizontalDistance < 2
      ? `M ${forkX} ${centerY} C ${forkX} ${centerY + direction * 28} ${forkX} ${laneY - direction * 22} ${forkX} ${laneY}`
      : `M ${forkX} ${centerY} C ${forkX} ${centerY + direction * 28} ${bendX} ${laneY - direction * 22} ${bendX} ${laneY} L ${headX} ${laneY}`;
    paths.push({
      id: `path-${branch.name}`,
      branchName: branch.name,
      status: branch.status,
      statusLabel: branch.statusLabel,
      severity: branch.severity,
      color: branch.color,
      d: pathD,
      isDefault: false,
      isFork: true,
      ahead: branch.ahead,
      behind: branch.behind,
      active: !activeBranchName || activeBranchName === branch.name,
      muted: isPathMuted(branch.name),
    });

    labels.push({
      id: `label-${branch.name}`,
      branchName: branch.name,
      status: branch.status,
      statusLabel: branch.statusLabel,
      color: branch.color,
      isDefault: false,
      ahead: branch.ahead,
      behind: branch.behind,
      health: branch.health,
      x: clampNumber(headX + 14, 16, width - 190),
      y: clampNumber(laneY - 10, 22, height - 46),
    });
  });

  return {
    paths,
    labels,
    heatZones: [],
    axisTicks: buildTimelineTicks(minUnit, maxUnit, unitToX, height),
    legendItems: normalizedBranches
      .slice()
      .sort((a, b) => Number(b.isDefault) - Number(a.isDefault) || a.sourceIndex - b.sourceIndex)
      .map((branch) => ({
        id: branch.id,
        branchName: branch.name,
        status: branch.status,
        statusLabel: branch.statusLabel,
        color: branch.color,
        isDefault: branch.isDefault,
        ahead: branch.ahead,
        behind: branch.behind,
        latestAt: branch.latestAt,
        active: !activeBranchName || activeBranchName === branch.name,
        muted: isPathMuted(branch.name),
      })),
    width,
    height,
    centerY,
    defaultBranchName: defaultName,
  };
}

function RecentDagModifications({ events }) {
  if (!events.length) {
    return null;
  }

  return (
    <section className="branch-dag-log" aria-label="Recent DAG modifications">
      <header className="branch-dag-log-header">
        <span>Recent DAG Modifications</span>
        <span>Filter by SHA...</span>
      </header>
      <div className="branch-dag-log-list">
        {events.map((event, index) => (
          <article className={`branch-dag-log-row tone-${event.tone}`} key={`${event.hash}-${index}`}>
            <span className="branch-dag-log-marker" aria-hidden="true" />
            <code>{shortHash(event.hash)}</code>
            <p>{event.message}</p>
            <time dateTime={event.created_at}>{formatRelativeTime(event.created_at)}</time>
          </article>
        ))}
      </div>
    </section>
  );
}

export function BranchGraph({ getToken, repository }) {
  const [hoveredBranchName, setHoveredBranchName] = useState(null);
  const graphPanelRef = useRef(null);
  const graphStageRef = useRef(null);

  const analyticsState = useRepositoryAnalytics({
    getToken,
    repoId: repository?.id,
    repository,
  });

  const data = analyticsState.data || {};
  const branches = Array.isArray(data.branches) ? data.branches : [];
  const topology = useMemo(
    () => buildDivergenceTopology({ branches, repository, hoveredBranchName }),
    [branches, repository, hoveredBranchName],
  );
  const metrics = useBranchMetrics({ analytics: data.analytics, branches });
  const loading = analyticsState.status === "loading";

  const paths = Array.isArray(topology.paths) ? topology.paths : [];
  const labels = Array.isArray(topology.labels) ? topology.labels : [];
  const heatZones = Array.isArray(topology.heatZones) ? topology.heatZones : [];
  const axisTicks = Array.isArray(topology.axisTicks) ? topology.axisTicks : [];
  const legendItems = Array.isArray(topology.legendItems) ? topology.legendItems : [];
  const topologyWidth = firstNumber(topology.width) || 1040;
  const topologyHeight = firstNumber(topology.height) || 430;
  const centerY = firstNumber(topology.centerY) || topologyHeight / 2;
  const hasTopology = branches.length > 0 && paths.length > 0;

  const analytics = data.analytics || {};
  const totalCommits = firstNumber(
    analytics.totalCommits,
    analytics.total_commits,
    analytics.commitCount,
    analytics.commit_count,
    branches.reduce((total, branch) => total + (firstNumber(branch.commitCount, branch.commit_count, branch.totalCommits) || 0), 0),
  );
  const longestChain = firstNumber(
    analytics.longestChain,
    analytics.longest_chain,
    analytics.deepestBranchPath,
    Math.max(0, ...branches.map((branch) => firstNumber(branch.commitCount, branch.commit_count, branch.divergence?.distance) || 0)),
  );
  const openPullRequests = firstNumber(
    analytics.openPullRequests,
    analytics.open_pull_requests,
    analytics.pullRequestsOpen,
    data.pullRequests?.open,
    data.pullRequests?.length,
  );
  const derivedComplexity = branches.length > 0
    ? Math.min(0.99, Math.max(0.1, (paths.length + branches.length) / Math.max(8, labels.length + branches.length)))
    : 0;
  const dagComplexity = normalizeComplexity(
    analytics.dagComplexity || analytics.dag_complexity || analytics.commitDagComplexity || analytics.commit_dag_complexity,
    derivedComplexity,
  );
  const complexityLabel = dagComplexity >= 0.75 ? "Stable" : dagComplexity >= 0.45 ? "Moderate" : "Low signal";
  const recentDagEvents = useMemo(() => buildRecentDagEvents(data, branches), [data, branches]);

  function handleFullScreen() {
    const target = graphPanelRef.current;
    if (target?.requestFullscreen) {
      target.requestFullscreen();
    }
  }

  function handleExportSvg() {
    const svg = graphStageRef.current?.querySelector("svg");
    if (!svg || typeof XMLSerializer === "undefined") {
      return;
    }

    const clone = svg.cloneNode(true);
    clone.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    clone.setAttribute("width", String(topologyWidth));
    clone.setAttribute("height", String(topologyHeight));

    const source = new XMLSerializer().serializeToString(clone);
    const blob = new Blob([source], { type: "image/svg+xml;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `${repositorySlug(repository)}-branch-divergence.svg`;
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
  }

  return (
    <section className="workspace-section branch-dashboard">
      <style>{BRANCH_DASHBOARD_STYLE}</style>

      <div className="branch-kpi-grid" aria-label="Repository branch KPIs">
        <article className="branch-kpi-card">
          <div className="branch-kpi-heading">
            <span>Commit DAG Complexity</span>
            <span className="material-symbols-outlined" aria-hidden="true">hub</span>
          </div>
          <div className="branch-kpi-value-row">
            <strong>{formatDecimal(dagComplexity, 2)}</strong>
            <span className="success">{complexityLabel}</span>
          </div>
          <div className="branch-kpi-progress" aria-hidden="true">
            <span style={{ width: `${Math.round(dagComplexity * 100)}%` }} />
          </div>
        </article>

        <article className="branch-kpi-card">
          <div className="branch-kpi-heading">
            <span>Longest Chain</span>
            <span className="material-symbols-outlined tertiary" aria-hidden="true">account_tree</span>
          </div>
          <div className="branch-kpi-value-row">
            <strong>{formatNumber(longestChain)}</strong>
            <span>nodes</span>
          </div>
          <p>Deepest branch path since merge</p>
        </article>

        <article className="branch-kpi-card">
          <div className="branch-kpi-heading">
            <span>Open Pull Requests</span>
            <span className="material-symbols-outlined" aria-hidden="true">merge_type</span>
          </div>
          <div className="branch-kpi-value-row">
            <strong>{formatNumber(openPullRequests)}</strong>
            <span>{branches.length} tracked branches</span>
          </div>
          <div className="branch-kpi-segments" aria-hidden="true">
            <span className="active" />
            <span className="active" />
            <span />
            <span />
          </div>
        </article>
      </div>

      <div className="vcs-observability-layout">
        <BranchSidebar branches={branches} defaultBranchName={topology.defaultBranchName} hoveredBranchName={hoveredBranchName} loading={loading} onHover={setHoveredBranchName} />

        <section className="vcs-graph-panel" ref={graphPanelRef}>
          <header className="vcs-panel-header">
            <div>
              <h2>Branch Divergence Visualization</h2>
            </div>
            <div className="vcs-panel-actions">
              <button type="button" onClick={handleFullScreen}>Full Screen</button>
              <button type="button" onClick={handleExportSvg}>Export SVG</button>
            </div>
          </header>

          <div className="branch-graph-stage" ref={graphStageRef}>
            {hasTopology ? (
              <>
                <svg
                  className="branch-topology-svg"
                  viewBox={`0 0 ${topologyWidth} ${topologyHeight}`}
                  preserveAspectRatio="xMidYMid meet"
                  role="img"
                  aria-label="Commit topology graph"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <defs>
                    <pattern id="topology-grid" width="72" height="72" patternUnits="userSpaceOnUse">
                      <path d="M 72 0 L 0 0 0 72" fill="none" stroke="rgba(139,145,157,0.10)" strokeWidth="1" />
                      <path d="M 0 36 L 72 36 M 36 0 L 36 72" fill="none" stroke="rgba(139,145,157,0.045)" strokeWidth="1" />
                    </pattern>
                    {paths.map((path) => (
                      <linearGradient id={`gradient-${gradientId(path.id)}`} key={path.id} x1="0%" x2="100%" y1="0%" y2="0%">
                        <stop offset="0%" stopColor="rgba(139,145,157,0.58)" />
                        <stop offset="55%" stopColor={path.color || "#58a6ff"} stopOpacity="0.9" />
                        <stop offset="100%" stopColor={path.color || "#58a6ff"} />
                      </linearGradient>
                    ))}
                  </defs>
                  <style>{BRANCH_GRAPH_SVG_STYLE}</style>
                  <rect className="branch-svg-background" width={topologyWidth} height={topologyHeight} />
                  <rect width={topologyWidth} height={topologyHeight} fill="url(#topology-grid)" />
                  <g className="graph-heat-layer">
                    {heatZones.map((zone) => (
                      <rect
                        height={zone.height}
                        key={zone.id}
                        rx="22"
                        style={{ fill: zone.color, opacity: zone.opacity }}
                        width={zone.width}
                        x={zone.x}
                        y={zone.y}
                      />
                    ))}
                  </g>
                  <g className="graph-baseline">
                    {axisTicks.map((tick) => (
                      <g key={tick.id}>
                        <line className="timeline-tick" x1={tick.x} x2={tick.x} y1="44" y2={topologyHeight - 42} />
                        <text className="timeline-tick-label" x={tick.x} y={tick.y}>{tick.label}</text>
                      </g>
                    ))}
                    <line x1="44" x2={topologyWidth - 44} y1={topologyHeight - 42} y2={topologyHeight - 42} />
                  </g>
                  <g className="graph-paths">
                    {paths.map((path) => (
                      <BranchPath key={path.id} onHover={setHoveredBranchName} path={path} />
                    ))}
                  </g>
                  <g className="branch-label-layer">
                    {labels.map((label) => (
                      <BranchLabel
                        hovered={hoveredBranchName === label.branchName}
                        key={label.id || label.branchName}
                        label={label}
                        onHover={setHoveredBranchName}
                      />
                    ))}
                  </g>
                </svg>
              </>
            ) : (
              <div className="vcs-graph-empty">
                <span className="material-symbols-outlined" aria-hidden="true">account_tree</span>
                <p>{loading ? "Building repository topology..." : analyticsState.error || "No topology data available"}</p>
              </div>
            )}
          </div>

          <BranchLegend branches={legendItems} hoveredBranchName={hoveredBranchName} onHover={setHoveredBranchName} />
          <BranchMetrics metrics={metrics} />
        </section>
      </div>

      <RecentDagModifications events={recentDagEvents} />
    </section>
  );
}

const BRANCH_GRAPH_SVG_STYLE = `
  .branch-svg-background { fill: #0b0e14; }
  .graph-heat-layer { filter: blur(18px); opacity: 0.22; }
  .graph-baseline line { stroke: #30363d; stroke-width: 3; stroke-linecap: round; }
  .graph-baseline .timeline-tick { stroke: rgba(139,145,157,0.18); stroke-width: 1; stroke-dasharray: 4 8; }
  .timeline-tick-label { fill: #8b919d; font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 10px; text-anchor: middle; }
  .branch-path-layer { cursor: pointer; outline: none; transition: opacity 150ms ease; }
  .branch-path-hitbox { stroke-width: 24; pointer-events: stroke; }
  .branch-path-shadow { fill: none; stroke: rgba(11,14,20,0.92); stroke-width: 11; stroke-linecap: round; stroke-linejoin: round; }
  .branch-path { fill: none; stroke-width: 4; stroke-linecap: round; stroke-linejoin: round; filter: drop-shadow(0 0 7px rgba(88,166,255,0.22)); transition: stroke-width 150ms ease, filter 150ms ease, opacity 150ms ease; }
  .branch-path-layer.default .branch-path { stroke: #c0c7d4; stroke-width: 5.4; filter: none; }
  .branch-path-layer.muted { opacity: 0.24; }
  .branch-path-layer.active .branch-path,
  .branch-path-layer:focus .branch-path { stroke-width: 6; filter: drop-shadow(0 0 12px rgba(88,166,255,0.34)); }
  .branch-path-layer.default.active .branch-path,
  .branch-path-layer.default:focus .branch-path { stroke-width: 7; }
  .branch-path-layer.status-idle .branch-path { stroke-dasharray: 12 6; }
  .branch-path-layer.status-outdated .branch-path,
  .branch-path-layer.status-stale .branch-path { stroke-dasharray: 3 8; }
  .branch-svg-label { cursor: pointer; outline: none; pointer-events: auto; transition: opacity 150ms ease; }
  .branch-svg-label-pill { fill: rgba(16,20,25,0.78); stroke: rgba(65,71,82,0.88); stroke-width: 1; }
  .branch-svg-label.hovered .branch-svg-label-pill,
  .branch-svg-label:focus .branch-svg-label-pill { stroke: var(--branch-color, #58a6ff); }
  .branch-svg-label-dot { fill: var(--branch-color, #58a6ff); }
  .branch-svg-label-name { fill: var(--branch-color, #a2c9ff); font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 11px; font-weight: 700; }
  .branch-svg-label-meta { fill: #c0c7d4; font-family: Inter, ui-sans-serif, system-ui, sans-serif; font-size: 9.5px; }
`;

const BRANCH_DASHBOARD_STYLE = `
  .branch-dashboard {
    --vcs-background: #101419;
    --vcs-surface: #101419;
    --vcs-surface-lowest: #0b0e14;
    --vcs-surface-low: #181c21;
    --vcs-surface-container: #1c2025;
    --vcs-surface-high: #272a30;
    --vcs-border: #414752;
    --vcs-border-soft: #30363d;
    --vcs-text: #e0e2ea;
    --vcs-muted: #c0c7d4;
    --vcs-subtle: #8b919d;
    --vcs-primary: #58a6ff;
    --vcs-primary-soft: #a2c9ff;
    --vcs-tertiary: #ffba42;
    --vcs-success: #3fb950;
    --vcs-danger: #ffb4ab;
    color: var(--vcs-text);
    font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }

  .branch-dashboard * { box-sizing: border-box; }
  .branch-dashboard code,
  .branch-dashboard .mono-font { font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }

  .branch-kpi-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
    margin-bottom: 16px;
  }

  .branch-kpi-card {
    min-height: 116px;
    border: 1px solid var(--vcs-border);
    border-radius: 8px;
    background: var(--vcs-surface-container);
    padding: 16px;
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.025);
  }

  .branch-kpi-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
    color: var(--vcs-muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.055em;
    line-height: 16px;
    text-transform: uppercase;
  }

  .branch-kpi-heading .material-symbols-outlined { color: var(--vcs-primary-soft); font-size: 25px; }
  .branch-kpi-heading .material-symbols-outlined.tertiary { color: var(--vcs-tertiary); }

  .branch-kpi-value-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .branch-kpi-value-row strong {
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 25px;
    font-weight: 700;
    letter-spacing: -0.04em;
    line-height: 32px;
  }

  .branch-kpi-value-row span,
  .branch-kpi-card p {
    margin: 0;
    color: var(--vcs-muted);
    font-size: 12px;
    line-height: 18px;
  }

  .branch-kpi-value-row span.success { color: var(--vcs-success); font-weight: 700; }

  .branch-kpi-progress {
    height: 4px;
    width: 100%;
    margin-top: 8px;
    overflow: hidden;
    border-radius: 999px;
    background: #161b22;
  }

  .branch-kpi-progress span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--vcs-primary-soft);
    box-shadow: 0 0 16px rgba(88,166,255,0.28);
  }

  .branch-kpi-segments {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 6px;
    margin-top: 10px;
  }

  .branch-kpi-segments span {
    height: 4px;
    border-radius: 999px;
    background: var(--vcs-border);
  }

  .branch-kpi-segments span.active { background: var(--vcs-primary-soft); }

  .vcs-observability-layout {
    display: grid;
    grid-template-columns: minmax(300px, 5fr) minmax(420px, 7fr);
    gap: 16px;
    align-items: stretch;
  }

  .vcs-branch-sidebar,
  .vcs-graph-panel,
  .branch-dag-log {
    overflow: hidden;
    border: 1px solid var(--vcs-border);
    border-radius: 8px;
    background: var(--vcs-surface-container);
  }

  .vcs-branch-sidebar,
  .vcs-graph-panel {
    display: flex;
    min-height: 624px;
    flex-direction: column;
  }

  .vcs-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 56px;
    border-bottom: 1px solid var(--vcs-border);
    background: var(--vcs-surface-low);
    padding: 14px 16px;
  }

  .vcs-panel-header h2 {
    margin: 0;
    color: var(--vcs-text);
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.01em;
    line-height: 24px;
  }

  .vcs-count-pill {
    white-space: nowrap;
    color: var(--vcs-muted);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 11px;
    line-height: 16px;
  }

  .vcs-panel-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px;
  }

  .vcs-panel-actions button {
    border: 1px solid var(--vcs-border);
    border-radius: 4px;
    background: #161b22;
    color: var(--vcs-text);
    cursor: pointer;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 11px;
    line-height: 16px;
    padding: 6px 10px;
    transition: background 150ms ease, border-color 150ms ease, color 150ms ease;
  }

  .vcs-panel-actions button:hover,
  .vcs-panel-actions button:focus-visible {
    border-color: var(--vcs-primary);
    background: #1c2128;
    color: var(--vcs-primary-soft);
    outline: none;
  }

  .branch-graph-stage {
    position: relative;
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    min-height: 500px;
    overflow: hidden;
    background: var(--vcs-surface-lowest);
  }

  .branch-topology-svg {
    display: block;
    width: 100%;
    height: 100%;
    min-height: 500px;
  }

  .vcs-graph-empty {
    display: grid;
    place-items: center;
    gap: 12px;
    min-height: 500px;
    color: var(--vcs-subtle);
    text-align: center;
  }

  .vcs-graph-empty .material-symbols-outlined { color: var(--vcs-primary); font-size: 32px; }
  .vcs-graph-empty p { margin: 0; font-size: 13px; }


  .branch-divergence-legend {
    border-top: 1px solid var(--vcs-border);
    background: #101419;
    padding: 12px 16px 14px;
  }

  .branch-divergence-legend-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 10px;
    color: var(--vcs-muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.045em;
    line-height: 16px;
    text-transform: uppercase;
  }

  .branch-divergence-legend-header span:last-child {
    color: var(--vcs-subtle);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-weight: 500;
    letter-spacing: 0;
    text-transform: none;
  }

  .branch-divergence-legend-list {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 8px;
  }

  .branch-divergence-legend-item {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    border: 1px solid rgba(65,71,82,0.72);
    border-radius: 6px;
    background: rgba(28,32,37,0.72);
    color: var(--vcs-text);
    cursor: pointer;
    padding: 9px 10px;
    text-align: left;
    transition: background 150ms ease, border-color 150ms ease, opacity 150ms ease, transform 150ms ease;
  }

  .branch-divergence-legend-item:hover,
  .branch-divergence-legend-item:focus-visible,
  .branch-divergence-legend-item.active {
    border-color: var(--branch-color, var(--vcs-primary));
    background: rgba(39,42,48,0.94);
    outline: none;
  }

  .branch-divergence-legend-item.muted { opacity: 0.46; }

  .branch-divergence-legend-swatch {
    width: 10px;
    height: 32px;
    border-radius: 999px;
    background: var(--branch-color, var(--vcs-primary));
    box-shadow: 0 0 14px color-mix(in srgb, var(--branch-color, var(--vcs-primary)) 38%, transparent);
  }

  .branch-divergence-legend-main {
    min-width: 0;
  }

  .branch-divergence-legend-name {
    display: block;
    min-width: 0;
    overflow: hidden;
    color: var(--branch-color, var(--vcs-primary-soft));
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    font-weight: 700;
    line-height: 16px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .branch-divergence-legend-meta {
    display: block;
    color: var(--vcs-muted);
    font-size: 11px;
    line-height: 16px;
  }

  .branch-divergence-status {
    border: 1px solid rgba(65,71,82,0.9);
    border-radius: 999px;
    color: var(--vcs-muted);
    font-size: 10px;
    font-weight: 800;
    line-height: 16px;
    padding: 1px 8px;
    text-transform: uppercase;
    white-space: nowrap;
  }

  .branch-divergence-status.status-active { border-color: rgba(88,166,255,0.32); color: var(--vcs-primary-soft); }
  .branch-divergence-status.status-idle { border-color: rgba(190,199,210,0.24); color: var(--vcs-muted); }
  .branch-divergence-status.status-outdated { border-color: rgba(255,186,66,0.34); color: var(--vcs-tertiary); }
  .branch-divergence-status.status-default { border-color: rgba(139,145,157,0.42); color: var(--vcs-text); }

  .branch-metrics-row {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 16px;
    border-top: 1px solid var(--vcs-border);
    background: var(--vcs-surface-low);
    padding: 16px;
  }

  .branch-metric span {
    display: block;
    margin-bottom: 4px;
    color: var(--vcs-muted);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.035em;
    line-height: 14px;
    text-transform: uppercase;
  }

  .branch-metric strong {
    color: var(--metric-accent, var(--vcs-text));
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 14px;
    font-weight: 600;
    line-height: 20px;
  }

  .vcs-branch-list {
    flex: 1;
    overflow: auto;
  }

  .vcs-branch-row {
    position: relative;
    padding: 16px;
    border-bottom: 1px solid var(--vcs-border-soft);
    background: rgba(28,32,37,0.86);
    cursor: default;
    transition: background 150ms ease;
  }

  .vcs-branch-row:hover,
  .vcs-branch-row:focus-visible,
  .vcs-branch-row.hovered { background: var(--vcs-surface-high); outline: none; }
  .vcs-branch-row.hovered { box-shadow: inset 3px 0 0 var(--branch-accent, var(--vcs-primary)); }
  .vcs-branch-row.muted { opacity: 0.48; }
  .vcs-branch-row:last-child { border-bottom: 0; }

  .vcs-branch-row-heading {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .vcs-branch-row-heading .material-symbols-outlined {
    color: var(--branch-accent, var(--vcs-primary));
    font-size: 16px;
  }

  .vcs-branch-row-heading strong {
    min-width: 0;
    overflow: hidden;
    color: var(--branch-accent, var(--vcs-primary-soft));
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 14px;
    font-weight: 600;
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vcs-status-badge {
    border: 1px solid var(--vcs-border);
    border-radius: 4px;
    background: rgba(65,71,82,0.38);
    color: var(--vcs-muted);
    font-size: 11px;
    font-weight: 800;
    line-height: 16px;
    padding: 2px 8px;
  }

  .vcs-status-badge.branch-status-active { border-color: rgba(88,166,255,0.32); background: rgba(88,166,255,0.10); color: var(--vcs-primary-soft); }
  .vcs-status-badge.branch-status-idle { border-color: rgba(190,199,210,0.24); background: rgba(190,199,210,0.10); color: var(--vcs-muted); }
  .vcs-status-badge.branch-status-outdated,
  .vcs-status-badge.branch-status-stale { border-color: rgba(255,186,66,0.32); background: rgba(255,186,66,0.10); color: var(--vcs-tertiary); }
  .vcs-status-badge.branch-status-default { border-color: rgba(139,145,157,0.40); background: rgba(65,71,82,0.72); color: var(--vcs-text); }

  .vcs-branch-metrics {
    display: flex;
    gap: 18px;
    margin-bottom: 12px;
  }

  .vcs-branch-metrics span {
    display: grid;
    gap: 2px;
  }

  .vcs-branch-metrics small {
    color: var(--vcs-muted);
    font-size: 10px;
    letter-spacing: 0.08em;
    line-height: 13px;
  }

  .vcs-branch-metrics strong {
    color: var(--vcs-text);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 14px;
    font-weight: 600;
    line-height: 18px;
  }

  .vcs-branch-metrics .ahead strong { color: var(--vcs-success); }
  .vcs-branch-metrics .behind strong { color: var(--vcs-danger); }

  .vcs-branch-row-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    color: var(--vcs-muted);
    font-size: 11px;
    line-height: 16px;
  }

  .vcs-branch-row-footer span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vcs-empty-state {
    margin: 0;
    padding: 18px 16px;
    color: var(--vcs-muted);
    font-size: 12px;
  }

  .branch-dag-log {
    margin-top: 16px;
  }

  .branch-dag-log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid var(--vcs-border);
    background: var(--vcs-surface-low);
    padding: 9px 16px;
    color: var(--vcs-muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.045em;
    line-height: 16px;
    text-transform: uppercase;
  }

  .branch-dag-log-header span:last-child {
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-weight: 500;
    letter-spacing: 0;
    text-transform: none;
  }

  .branch-dag-log-row {
    display: grid;
    grid-template-columns: auto auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 14px;
    border-bottom: 1px solid var(--vcs-border-soft);
    padding: 10px 16px;
    transition: background 150ms ease;
  }

  .branch-dag-log-row:hover { background: var(--vcs-surface-high); }
  .branch-dag-log-row:last-child { border-bottom: 0; }

  .branch-dag-log-marker {
    width: 4px;
    height: 24px;
    border-radius: 999px;
    background: var(--vcs-success);
  }

  .branch-dag-log-row.tone-danger .branch-dag-log-marker { background: var(--vcs-danger); }
  .branch-dag-log-row.tone-warn .branch-dag-log-marker { background: var(--vcs-tertiary); }

  .branch-dag-log-row code {
    color: var(--vcs-primary-soft);
    font-size: 11px;
    line-height: 16px;
  }

  .branch-dag-log-row p {
    min-width: 0;
    margin: 0;
    overflow: hidden;
    color: var(--vcs-text);
    font-size: 13px;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .branch-dag-log-row time {
    color: var(--vcs-muted);
    font-size: 11px;
    line-height: 16px;
    white-space: nowrap;
  }

  @media (max-width: 1180px) {
    .vcs-observability-layout { grid-template-columns: 1fr; }
    .vcs-branch-sidebar,
    .vcs-graph-panel { min-height: auto; }
  }

  @media (max-width: 780px) {
    .branch-kpi-grid { grid-template-columns: 1fr; }
    .branch-metrics-row { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .branch-dag-log-row { grid-template-columns: auto auto minmax(0, 1fr); }
    .branch-dag-log-row time { display: none; }
  }
`;
