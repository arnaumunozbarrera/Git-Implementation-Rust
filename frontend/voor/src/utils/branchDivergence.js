export function computeBranchDivergence(branchGraph, defaultGraph) {
  const branchNodes = Array.isArray(branchGraph?.nodes) ? branchGraph.nodes : [];
  const defaultNodes = Array.isArray(defaultGraph?.nodes) ? defaultGraph.nodes : [];
  const defaultHashes = new Set(defaultNodes.map((node) => node.hash));
  const branchHashes = new Set(branchNodes.map((node) => node.hash));
  const common = branchNodes.find((node) => defaultHashes.has(node.hash));
  const ahead = common
    ? branchNodes.findIndex((node) => node.hash === common.hash)
    : branchNodes.length;
  const behind = defaultNodes.filter((node) => !branchHashes.has(node.hash)).length;

  return {
    ahead: Math.max(0, ahead),
    behind: Math.max(0, behind),
    commonHash: common?.hash || null,
    distance: Math.max(0, ahead) + Math.max(0, behind),
  };
}

export function severityForDivergence(distance, status) {
  if (status === "default") {
    return "neutral";
  }

  if (status === "outdated" || distance >= 18) {
    return "critical";
  }

  if (status === "idle" || distance >= 8) {
    return "warning";
  }

  return "healthy";
}
