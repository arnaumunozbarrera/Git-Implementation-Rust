import { useMemo } from "react";

function compactNumber(value) {
  return new Intl.NumberFormat("en", { notation: "compact" }).format(Number(value) || 0);
}

export function useBranchMetrics({ analytics, branches }) {
  return useMemo(() => {
    const branchList = Array.isArray(branches) ? branches : [];
    const nonDefault = branchList.filter((branch) => branch.status !== "default");
    const staleCount = branchList.filter((branch) => branch.status === "outdated").length;
    const totalDistance = branchList.reduce((sum, branch) => sum + (branch.divergence?.distance || 0), 0);
    const avgDivergence = branchList.length ? totalDistance / branchList.length : 0;
    const activeBranches = branchList.filter((branch) => branch.status === "active" || branch.status === "default").length;
    const mergeVelocity = branchList.length ? (activeBranches / branchList.length) * 12 : 0;

    return [
      {
        label: "TOTAL COMMITS",
        value: compactNumber(analytics?.commits_count || branchList.reduce((sum, branch) => sum + (branch.commitCount || 0), 0)),
        accent: "#a5d6ff",
      },
      {
        label: "AVG DIVERGENCE",
        value: `${avgDivergence.toFixed(1)} nodes`,
        accent: "#d2a8ff",
      },
      {
        label: "STALE RATIO",
        value: `${branchList.length ? ((staleCount / Math.max(1, nonDefault.length || branchList.length)) * 100).toFixed(1) : "0.0"}%`,
        accent: "#ffba42",
      },
      {
        label: "MERGE VELOCITY",
        value: `${mergeVelocity.toFixed(1)} PR/wk`,
        accent: "#7ee787",
      },
    ];
  }, [analytics, branches]);
}
