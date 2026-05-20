const graphWidth = 1040;
const graphHeight = 430;
const padding = { left: 58, right: 46, top: 56, bottom: 66 };

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
  const step = (graphWidth - padding.left - padding.right) / Math.max(8, span + 3);
  const centerY = graphHeight / 2;
  const nodeMap = new Map();
  const paths = [];
  const heatZones = [];

  defaultNodes.forEach((node, index) => {
    const x = padding.left + index * step;
    const y = centerY;
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
        d: defaultNodes
          .map((node, index) => `${index === 0 ? "M" : "L"} ${padding.left + index * step} ${centerY}`)
          .join(" "),
        active: active || !selectedBranchName,
        muted,
        severity: branch.severity,
      });
      return;
    }

    pathPoints.push({ x: startX, y: centerY });
    branchOnly.forEach((node, index) => {
      const x = startX + (index + 1.15) * step * 1.18;
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

    if (pathPoints.length === 1 && graph.head) {
      const existing = nodeMap.get(graph.head);
      if (existing) {
        existing.branchNames.add(branch.name);
        existing.isHead = true;
      }
    }

    const [first, ...rest] = pathPoints;
    const d = rest.reduce((path, point, index) => {
      const previous = pathPoints[index];
      const controlX = previous.x + (point.x - previous.x) * 0.55;
      return `${path} C ${controlX} ${previous.y}, ${controlX} ${point.y}, ${point.x} ${point.y}`;
    }, `M ${first.x} ${first.y}`);

    paths.push({
      id: branch.name,
      branchName: branch.name,
      color: branch.accent,
      d,
      active,
      muted,
      severity: branch.severity,
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
  };
}
