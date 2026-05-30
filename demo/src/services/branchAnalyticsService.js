import { fetchActivityFeed, fetchAnalyticsOverview, fetchBranchAnalytics, fetchBranches, fetchCommitGraph, fetchVcsAnalytics } from "../api.js";
import { normalizeBranchAnalytics } from "../utils/topologyBuilder.js";
import { severityForDivergence } from "../utils/branchDivergence.js";

const graphLimit = 48;
const branchGraphLimit = 10;

const branchPalette = ["#58a6ff", "#ffba42", "#7ee787", "#d2a8ff", "#39c5cf", "#ffa657", "#ff7b72", "#a5d6ff"];

function normalizeFreshnessStatus(branch) {
  if (branch.is_default) {
    return "default";
  }

  const status = String(branch.freshness_status || "").trim().toLowerCase();
  if (["active", "idle", "outdated"].includes(status)) {
    return status;
  }

  const staleDays = Number(branch.stale_days);
  if (Number.isFinite(staleDays)) {
    if (staleDays < 15) return "active";
    if (staleDays <= 30) return "idle";
  }

  return "outdated";
}

function displayName(author) {
  return author?.username || author?.email || author?.id || "";
}

function normalizeVcsBranch(branch, index) {
  const status = normalizeFreshnessStatus(branch);
  const divergence = {
    ahead: Number(branch.ahead_count) || 0,
    behind: Number(branch.behind_count) || 0,
    commonHash: branch.merge_base_hash || null,
    distance: Number(branch.divergence_distance) || 0,
  };
  const severity = severityForDivergence(divergence.distance, status);
  const latestCommit = branch.latest_commit || null;

  return {
    ...branch,
    id: branch.id || `${branch.repo_id || "repo"}:${branch.name}`,
    last_commit_hash: branch.last_commit_hash || branch.head_commit_hash || "",
    latestCommit,
    latestContributor: displayName(latestCommit?.author),
    latestMessage: latestCommit?.message || "",
    commitCount: Math.max(0, Math.round(Number(branch.commit_count ?? branch.commit_density) || 0)),
    activityScore: Number(branch.activity_score) || 0,
    accent: branch.lane_color || branchPalette[index % branchPalette.length],
    status,
    severity,
    divergence,
  };
}

function graphFromTopologyCache(cacheItem) {
  if (!Array.isArray(cacheItem?.nodes) || cacheItem.nodes.length === 0) {
    return null;
  }

  return {
    repo_id: cacheItem.repo_id || "",
    ref: cacheItem.branch_name,
    head: cacheItem.head_commit_hash,
    nodes: cacheItem.nodes,
  };
}

export async function fetchRepositoryAnalytics({ repoId, getToken, repository }) {
  if (!repoId) {
    return {
      activity: [],
      analytics: null,
      branches: [],
      graphsByBranch: {},
      timeline: [],
    };
  }

  const vcsAnalytics = await fetchBranchAnalytics(repoId, getToken)
    .catch(() => fetchVcsAnalytics(repoId, getToken))
    .catch(() => null);
  if (vcsAnalytics?.branches) {
    const analyticsResponse = await fetchAnalyticsOverview(repoId, getToken).catch(() => null);
    const activityResponse = await fetchActivityFeed(repoId, getToken, 24).catch(() => ({ items: [] }));
    const branches = vcsAnalytics.branches.map(normalizeVcsBranch);
    const cachedGraphs = Object.fromEntries(
      (vcsAnalytics.topology_cache || [])
        .map((item) => [item.branch_name, graphFromTopologyCache(item)])
        .filter(([, graph]) => graph),
    );
    const graphBranches = branches
      .filter((branch) => !cachedGraphs[branch.name])
      .slice(0, branchGraphLimit);
    const graphPairs = await Promise.all(
      graphBranches.map((branch) =>
        fetchCommitGraph(repoId, branch.name, getToken, graphLimit)
          .then((graph) => [branch.name, graph])
          .catch(() => [branch.name, { nodes: [], head: branch.head_commit_hash || branch.last_commit_hash || "", ref: branch.name }]),
      ),
    );

    return {
      activity: Array.isArray(activityResponse?.items) ? activityResponse.items : [],
      analytics: { ...(analyticsResponse || {}), ...(vcsAnalytics.dag_metrics || {}) },
      branches,
      graphsByBranch: { ...cachedGraphs, ...Object.fromEntries(graphPairs) },
      timeline: vcsAnalytics.timeline || [],
    };
  }

  const [branchesResponse, analyticsResponse, activityResponse] = await Promise.all([
    fetchBranches(repoId, getToken),
    fetchAnalyticsOverview(repoId, getToken).catch(() => null),
    fetchActivityFeed(repoId, getToken, 24).catch(() => ({ items: [] })),
  ]);

  const branches = Array.isArray(branchesResponse) ? branchesResponse : [];
  const defaultName = repository?.default_branch;
  const prioritizedBranches = [...branches].sort((left, right) => {
    if (left.name === defaultName) return -1;
    if (right.name === defaultName) return 1;
    return String(left.name).localeCompare(String(right.name));
  });
  const graphBranches = prioritizedBranches.slice(0, branchGraphLimit);
  const graphPairs = await Promise.all(
    graphBranches.map((branch) =>
      fetchCommitGraph(repoId, branch.name, getToken, graphLimit)
        .then((graph) => [branch.name, graph])
        .catch(() => [branch.name, { nodes: [], head: branch.last_commit_hash || "", ref: branch.name }]),
    ),
  );
  const graphsByBranch = Object.fromEntries(graphPairs);
  const enrichedBranches = normalizeBranchAnalytics({
    branches,
    graphsByBranch,
    repository,
  });

  return {
    activity: Array.isArray(activityResponse?.items) ? activityResponse.items : [],
    analytics: analyticsResponse,
    branches: enrichedBranches,
    graphsByBranch,
    timeline: [],
  };
}
