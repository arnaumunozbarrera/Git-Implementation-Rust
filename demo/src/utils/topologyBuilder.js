import { computeBranchDivergence, severityForDivergence } from "./branchDivergence.js";
import { detectBranchStatus } from "./staleBranchDetection.js";

const branchPalette = ["#58a6ff", "#ffba42", "#7ee787", "#d2a8ff", "#39c5cf", "#ffa657", "#ff7b72", "#a5d6ff"];

function userName(author) {
  return author?.username || author?.email || author?.id || "";
}

function latestNode(graph) {
  return Array.isArray(graph?.nodes) ? graph.nodes[0] : null;
}

export function normalizeBranchAnalytics({ branches, graphsByBranch, repository }) {
  const defaultBranchName = repository?.default_branch || branches?.[0]?.name || "main";
  const defaultGraph = graphsByBranch[defaultBranchName] || graphsByBranch[branches?.[0]?.name] || {};

  return (branches || []).map((branch, index) => {
    const graph = graphsByBranch[branch.name] || {};
    const latestCommit = latestNode(graph);
    const enriched = { ...branch, latestCommit };
    const status = detectBranchStatus(enriched, defaultBranchName);
    const divergence = computeBranchDivergence(graph, defaultGraph);
    const severity = severityForDivergence(divergence.distance, status);

    return {
      ...enriched,
      accent: branchPalette[index % branchPalette.length],
      commitCount: Array.isArray(graph.nodes) ? graph.nodes.length : 0,
      divergence,
      latestContributor: userName(latestCommit?.author),
      latestMessage: latestCommit?.message || "",
      status,
      severity,
    };
  });
}
