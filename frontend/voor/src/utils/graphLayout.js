const graphWidth = 1040;
const graphHeight = 430;
const padding = { left: 70, right: 70, top: 54, bottom: 74 };

function chronological(nodes) {
  return [...(nodes || [])].reverse();
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
  const defaultGraph = graphsByBranch[defaultBranchName] || graphsByBranch[sortedBranches[0]?.name] || {};
  const defaultNodes = chronological(defaultGraph.nodes);
  const defaultHashes = new Map(defaultNodes.map((node, index) => [node.hash, index]));
  const span = Math.max(1, defaultNodes.length - 1);
  const step = (graphWidth - padding.left - padding.right) / Math.max(8, span + 2);
  const centerY = graphHeight / 2;
  const nodeMap = new Map();
  const paths = [];
  const heatZones = [];
  const labels = [];
  const defaultPoints = [];

  defaultNodes.forEach((node, index) => {
    const x = padding.left + index * step;
    const y = centerY;
    defaultPoints.push({ x, y });
    nodeMap.set(node.hash, {
      ...node,
      x,
      y,
      branchName: defaultBranchName,
      branchNames: new Set([defaultBranchName, ...(node.branches || [])]),
      isDefault: true,
      isHead: node.hash === defaultGraph.head,
      state: "default",
    });
  });

  sortedBranches.forEach((branch, branchIndex) => {
    const graph = graphsByBranch[branch.name] || {};
    const nodes = chronological(graph.nodes);
    const offset = laneOffset(branchIndex);
    const laneY = clamp(centerY + offset * 78, padding.top + 26, graphHeight - padding.bottom);
    const commonHash = branch.divergence?.commonHash;
    const commonIndex = commonHash && defaultHashes.has(commonHash) ? defaultHashes.get(commonHash) : Math.max(0, defaultNodes.length - 2);
    const branchOnly = nodes.filter((node) => !defaultHashes.has(node.hash));
    const startX = padding.left + commonIndex * step;
    const active = !selectedBranchName && !hoveredBranchName
      ? true
      : branch.name === selectedBranchName || branch.name === hoveredBranchName;
    const muted = Boolean(selectedBranchName || hoveredBranchName) && !active && branch.name !== defaultBranchName;
    const pathPoints = [];

    if (branch.name === defaultBranchName) {
      paths.push({
        id: branch.name,
        branchName: branch.name,
        color: branch.accent,
        d: straightPath(defaultPoints),
        active: active || !selectedBranchName,
        isDefault: true,
        muted,
        severity: branch.severity,
      });
      labels.push({
        id: branch.name,
        branchName: branch.name,
        x: defaultPoints.at(-1)?.x || graphWidth - padding.right,
        y: centerY - 42,
        color: branch.accent,
        status: branch.status,
        ahead: branch.divergence?.ahead || 0,
        behind: branch.divergence?.behind || 0,
        isDefault: true,
      });
      return;
    }

    pathPoints.push({ x: startX, y: centerY });
    const visibleBranchNodes = branchOnly.length > 0
      ? branchOnly
      : nodes.filter((node) => node.hash === graph.head).slice(0, 1);

    visibleBranchNodes.forEach((node, index) => {
      const x = Math.min(graphWidth - padding.right, startX + (index + 1.1) * step * 1.08);
      const intensity = Math.min(1, 0.22 + (branch.divergence?.distance || 0) / 18);
      const y = laneY + Math.sin(index * 0.85) * 8;
      pathPoints.push({ x, y });
      nodeMap.set(node.hash, {
        ...node,
        x,
        y,
        branchName: branch.name,
        branchNames: new Set([branch.name, ...(node.branches || [])]),
        isDefault: false,
        isHead: node.hash === graph.head || node.hash === branch.last_commit_hash,
        state: branch.status === "outdated" ? "stale" : "active",
        intensity,
      });
    });

    const branchEnd = pathPoints.at(-1) || pathPoints[0];
    const mergeTargetIndex = Math.min(defaultPoints.length - 1, Math.max(commonIndex + Math.max(2, visibleBranchNodes.length), commonIndex + 1));
    const mergeTarget = defaultPoints[mergeTargetIndex];
    const mergePoints = mergeTarget && branch.status !== "outdated" && (branch.divergence?.behind || 0) !== 0
      ? [branchEnd, { x: branchEnd.x + step * 0.65, y: branchEnd.y }, mergeTarget]
      : [];

    paths.push({
      id: branch.name,
      branchName: branch.name,
      color: branch.accent,
      d: pathThrough(pathPoints),
      active,
      muted,
      severity: branch.severity,
    });

    if (mergePoints.length > 0) {
      paths.push({
        id: `${branch.name}-merge`,
        branchName: branch.name,
        color: branch.accent,
        d: pathThrough(mergePoints),
        active,
        muted,
        isMerge: true,
        severity: branch.severity,
      });
    }

    labels.push({
      id: branch.name,
      branchName: branch.name,
      x: Math.min(graphWidth - 168, branchEnd.x + 16),
      y: clamp(branchEnd.y + (branchEnd.y < centerY ? -52 : 24), padding.top, graphHeight - padding.bottom + 14),
      color: branch.accent,
      status: branch.status,
      ahead: branch.divergence?.ahead || 0,
      behind: branch.divergence?.behind || 0,
      health: branch.health_score,
      isDefault: false,
    });

    heatZones.push({
      id: branch.name,
      x: startX + step,
      y: laneY - 28,
      width: Math.max(120, branchOnly.length * step * 1.12),
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
  };
}
