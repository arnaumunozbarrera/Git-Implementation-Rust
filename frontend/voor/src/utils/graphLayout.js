const graphWidth = 1040;
const graphHeight = 500;
const padding = { left: 70, right: 70, top: 54, bottom: 74 };

function chronological(nodes) {
  return [...(nodes || [])].sort((left, right) => {
    const leftTime = timestamp(left);
    const rightTime = timestamp(right);
    if (leftTime !== rightTime) {
      return leftTime - rightTime;
    }

    return (Number(right.depth_from_head) || 0) - (Number(left.depth_from_head) || 0);
  });
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function laneOffset(index) {
  if (index === 0) {
    return 0;
  }

  const ring = Math.ceil(index / 2);
  return (index % 2 === 0 ? 1 : -1) * ring;
}

function timestamp(node) {
  const value = node?.created_at || node?.committed_at || node?.date || node?.timestamp;
  const parsed = value ? new Date(value).getTime() : Number.NaN;
  return Number.isFinite(parsed) ? parsed : 0;
}

function formatAxisDate(value) {
  if (!value) {
    return "";
  }

  return new Intl.DateTimeFormat("en", { month: "short", day: "numeric" }).format(new Date(value));
}

function pathThrough(points) {
  if (points.length === 0) {
    return "";
  }

  return points.reduce((path, point, index) => {
    if (index === 0) {
      return `M ${point.x} ${point.y}`;
    }

    const previous = points[index - 1];
    const controlX = previous.x + (point.x - previous.x) * 0.52;
    return `${path} C ${controlX} ${previous.y}, ${controlX} ${point.y}, ${point.x} ${point.y}`;
  }, "");
}

function straightPath(points) {
  return points
    .map((point, index) => `${index === 0 ? "M" : "L"} ${point.x} ${point.y}`)
    .join(" ");
}

export function layoutTopology({ branches, graphsByBranch, repository, selectedBranchName, hoveredBranchName }) {
  const defaultBranchName = repository?.default_branch || branches?.[0]?.name || "main";
  const sortedBranches = [...(branches || [])].sort((left, right) => {
    if (left.name === defaultBranchName) return -1;
    if (right.name === defaultBranchName) return 1;
    if (Number.isFinite(Number(left.lane_index)) && Number.isFinite(Number(right.lane_index))) {
      return Number(left.lane_index) - Number(right.lane_index);
    }
    return (right.divergence?.distance || 0) - (left.divergence?.distance || 0);
  });
  const branchCount = Math.max(1, sortedBranches.length);
  const laneSpacing = Math.min(76, Math.max(44, (graphHeight - padding.top - padding.bottom) / Math.max(1, branchCount - 1)));
  const centerY = padding.top + Math.floor((branchCount - 1) / 2) * laneSpacing;
  const branchLaneY = new Map(sortedBranches.map((branch, index) => [branch.name, padding.top + index * laneSpacing]));
  const allGraphNodes = sortedBranches.flatMap((branch) => chronological(graphsByBranch[branch.name]?.nodes || []));
  const timedNodes = allGraphNodes.filter((node) => timestamp(node) > 0);
  const minTime = Math.min(...timedNodes.map(timestamp), Date.now());
  const maxTime = Math.max(...timedNodes.map(timestamp), minTime + 1);
  const timeSpan = Math.max(1, maxTime - minTime);
  const fallbackStep = (graphWidth - padding.left - padding.right) / Math.max(8, Math.max(...sortedBranches.map((branch) => (graphsByBranch[branch.name]?.nodes || []).length), 1));
  const hashPositions = new Map();
  const nodeMap = new Map();
  const paths = [];
  const heatZones = [];
  const labels = [];
  const axisTicks = [0, 0.25, 0.5, 0.75, 1].map((ratio) => {
    const time = minTime + timeSpan * ratio;
    return {
      id: `tick-${ratio}`,
      x: padding.left + ratio * (graphWidth - padding.left - padding.right),
      y: graphHeight - padding.bottom + 32,
      label: formatAxisDate(time),
    };
  });

  sortedBranches.forEach((branch, branchIndex) => {
    const graph = graphsByBranch[branch.name] || {};
    const nodes = chronological(graph.nodes);
    const laneY = branchLaneY.get(branch.name) ?? clamp(centerY + laneOffset(branchIndex) * 72, padding.top + 26, graphHeight - padding.bottom);
    const commonHash = branch.divergence?.commonHash || branch.merge_base_hash;
    const active = !selectedBranchName && !hoveredBranchName
      ? true
      : branch.name === selectedBranchName || branch.name === hoveredBranchName;
    const muted = Boolean(selectedBranchName || hoveredBranchName) && !active && branch.name !== defaultBranchName;
    const visibleBranchNodes = nodes.length > 0
      ? nodes
      : branch.head_commit_hash || branch.last_commit_hash
        ? [{
            hash: branch.head_commit_hash || branch.last_commit_hash,
            created_at: branch.latest_commit_at || branch.last_seen_at || branch.created_at,
            message: branch.latestMessage || branch.status_reason || "",
            branches: [branch.name],
          }]
        : [];
    const pathPoints = visibleBranchNodes.map((node, index) => {
      const nodeTime = timestamp(node);
      const x = nodeTime > 0
        ? padding.left + ((nodeTime - minTime) / timeSpan) * (graphWidth - padding.left - padding.right)
        : padding.left + index * fallbackStep;
      return {
        x: clamp(x, padding.left, graphWidth - padding.right),
        y: laneY,
      };
    });

    const mergeNode = commonHash ? visibleBranchNodes.find((node) => node.hash === commonHash) : null;
    const mergePosition = commonHash && hashPositions.has(commonHash)
      ? hashPositions.get(commonHash)
      : mergeNode
        ? pathPoints[visibleBranchNodes.indexOf(mergeNode)]
        : null;

    if (mergePosition && branch.name !== defaultBranchName && pathPoints.length > 0) {
      const firstPoint = pathPoints[0];
      paths.push({
        id: `${branch.name}-fork`,
        branchName: branch.name,
        color: branch.accent,
        d: pathThrough([mergePosition, { x: Math.max(mergePosition.x + 18, firstPoint.x - 24), y: laneY }, firstPoint]),
        active,
        muted,
        isFork: true,
        severity: branch.severity,
        status: branch.status,
      });
    }

    visibleBranchNodes.forEach((node, index) => {
      const intensity = Math.min(1, 0.22 + (branch.divergence?.distance || 0) / 18);
      const point = pathPoints[index];
      hashPositions.set(node.hash, point);
      const existing = nodeMap.get(node.hash);
      const branchNames = new Set([
        ...(existing?.branchNames || []),
        branch.name,
        ...(node.branches || []),
      ]);
      nodeMap.set(node.hash, {
        ...(existing || {}),
        ...node,
        x: existing?.x ?? point.x,
        y: existing?.y ?? point.y,
        branchName: branch.name,
        branchNames,
        isDefault: branch.name === defaultBranchName || node.is_default_branch,
        isHead: node.hash === graph.head || node.hash === branch.last_commit_hash,
        state: branch.status === "outdated" ? "stale" : "active",
        intensity,
      });
    });

    paths.push({
      id: branch.name,
      branchName: branch.name,
      color: branch.accent,
      d: straightPath(pathPoints),
      active: active || branch.name === defaultBranchName,
      isDefault: branch.name === defaultBranchName,
      muted,
      severity: branch.severity,
      status: branch.status,
    });

    const branchEnd = pathPoints.at(-1) || { x: padding.left, y: laneY };
    labels.push({
      id: branch.name,
      branchName: branch.name,
      x: Math.min(graphWidth - 168, branchEnd.x + 16),
      y: clamp(branchEnd.y - 25, padding.top - 26, graphHeight - padding.bottom + 14),
      color: branch.accent,
      status: branch.status,
      ahead: branch.divergence?.ahead || 0,
      behind: branch.divergence?.behind || 0,
      health: branch.health_score,
      isDefault: branch.name === defaultBranchName,
    });

    heatZones.push({
      id: branch.name,
      x: pathPoints[0]?.x || padding.left,
      y: laneY - 28,
      width: Math.max(120, (branchEnd.x - (pathPoints[0]?.x || padding.left)) + 36),
      height: 56,
      color: branch.accent,
      opacity: Math.min(0.16, Math.max(branch.status === "outdated" ? 0.04 : 0.08, Number(branch.activity_heat) || 0)),
    });
  });

  const nodes = [...nodeMap.values()].map((node) => ({
    ...node,
    branchNames: [...node.branchNames],
  }));

  return {
    width: graphWidth,
    height: graphHeight,
    centerY,
    branches: sortedBranches,
    paths,
    nodes,
    heatZones,
    labels,
    axisTicks,
  };
}
