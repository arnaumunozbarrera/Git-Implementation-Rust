const now = new Date("2026-12-31T12:00:00.000Z");
const developmentStart = new Date("2026-01-01T00:00:00.000Z");
const developmentDays = Math.floor((now.getTime() - developmentStart.getTime()) / 86400000) + 1;

const users = [
  { id: "demo_user_01", username: "Maya Chen", email: "maya.chen@example.invalid" },
  { id: "demo_user_02", username: "Noah Patel", email: "noah.patel@example.invalid" },
  { id: "demo_user_03", username: "Elena Garcia", email: "elena.garcia@example.invalid" },
  { id: "demo_user_04", username: "Owen Brooks", email: "owen.brooks@example.invalid" },
];

const repositories = [
  {
    id: "voor-platform",
    name: "voor-platform",
    owner_id: "demo_user_01",
    default_branch: "main",
    is_private: true,
    description: "Distributed VCS telemetry and branch analytics platform",
    created_at: "2026-01-06T09:18:00.000Z",
    updated_at: isoDaysAgo(0),
  },
  {
    id: "checkout-service",
    name: "checkout-service",
    owner_id: "demo_user_01",
    default_branch: "main",
    is_private: false,
    description: "Payment workflow reference repository",
    created_at: "2026-01-19T11:42:00.000Z",
    updated_at: isoDaysAgo(2),
  },
  {
    id: "infra-recipes",
    name: "infra-recipes",
    owner_id: "demo_user_01",
    default_branch: "trunk",
    is_private: true,
    description: "Deployment manifests and operational recipes",
    created_at: "2026-02-02T08:55:00.000Z",
    updated_at: isoDaysAgo(1),
  },
];

const branchNamesByRepo = {
  "voor-platform": ["main", "feature/topology-cache", "release/0.4", "fix/auth-refresh", "experiment/graph-layout", "docs/vercel-demo"],
  "checkout-service": ["main", "feature/refunds", "release/may", "fix/webhook-retry", "perf/cart-cache"],
  "infra-recipes": ["trunk", "cluster/blue", "cluster/green", "feature/audit-logs", "fix/secret-rotation"],
};

const filePools = {
  "voor-platform": [
    "src/App.jsx",
    "src/api.js",
    "src/components/vcs/branch-graph/BranchGraph.jsx",
    "src/components/AnalyticsOverviewCard.jsx",
    "src/hooks/useRepositoryAnalytics.js",
    "src/services/branchAnalyticsService.js",
    "src/utils/topologyBuilder.js",
    "server/api/services/frontend_service.rs",
    "server/api/services/sync_service.rs",
    "server/api/routes/repositories.rs",
    "backend/voor/src/commands/sync.rs",
    "src/styles.css",
    "README.md",
  ],
  "checkout-service": [
    "src/payments/checkout.ts",
    "src/payments/refunds.ts",
    "src/webhooks/stripe.ts",
    "src/cart/cache.ts",
    "src/orders/fulfillment.ts",
    "src/taxes/vat.ts",
    "tests/checkout.spec.ts",
    "tests/webhooks.spec.ts",
    "db/migrations/settlements.sql",
    "docs/runbook.md",
  ],
  "infra-recipes": [
    "clusters/prod/apps.yaml",
    "clusters/staging/apps.yaml",
    "clusters/dev/apps.yaml",
    "terraform/network/main.tf",
    "terraform/iam/policies.tf",
    "terraform/secrets/kms.tf",
    "scripts/rotate-secrets.ps1",
    "scripts/drain-node.ps1",
    "observability/alerts.yaml",
    "observability/dashboards/sync.json",
  ],
};

const commitMessages = [
  "Refine branch topology rendering",
  "Add sync monitor anomaly summary",
  "Tune repository dashboard queries",
  "Improve activity heatmap density",
  "Normalize remote action payloads",
  "Update failure propagation buckets",
  "Harden account profile validation",
  "Refresh Vercel deployment settings",
  "Improve commit graph pagination",
  "Adjust service health telemetry",
  "Add branch divergence scoring",
  "Stabilize pull request metrics",
  "Split dashboard refresh worker",
  "Backfill repository activity totals",
  "Fix branch graph hover state",
  "Rework stale branch thresholds",
  "Patch webhook retry ordering",
  "Add settlement export checks",
  "Document rollback procedure",
  "Tighten deployment diff output",
  "Reduce noisy sync monitor logs",
  "Handle empty topology cache",
  "Batch migration status lookups",
  "Improve error boundary copy",
];

const commitsByRepo = Object.fromEntries(repositories.map((repo, repoIndex) => [
  repo.id,
  buildCommits(repo, repoIndex),
]));

const branchesByRepo = Object.fromEntries(repositories.map((repo, repoIndex) => [
  repo.id,
  buildBranches(repo, repoIndex, commitsByRepo[repo.id]),
]));

const timelinesByRepo = Object.fromEntries(repositories.map((repo, repoIndex) => [
  repo.id,
  buildTimeline(repoIndex),
]));

function isoDaysAgo(days, hour = 10) {
  const date = new Date(now);
  date.setUTCDate(date.getUTCDate() - days);
  date.setUTCHours(hour, (days * 13) % 60, 0, 0);
  return date.toISOString();
}

function hashFor(repoId, index) {
  const seed = `${repoId}-${index}-demo`;
  let output = "";
  for (let i = 0; output.length < 40; i += 1) {
    const code = seed.charCodeAt(i % seed.length) + i * 17;
    output += (code % 16).toString(16);
  }
  return output;
}

function pick(items, index) {
  return items[index % items.length];
}

function dateFromStart(dayIndex, hour = 10, minute = 0) {
  const date = new Date(developmentStart);
  date.setUTCDate(date.getUTCDate() + dayIndex);
  date.setUTCHours(hour, minute, 0, 0);
  return date.toISOString();
}

function buildDailyActivity(repoIndex) {
  return Array.from({ length: developmentDays }, (_, dayIndex) => {
    const date = new Date(developmentStart);
    date.setUTCDate(date.getUTCDate() + dayIndex);
    const weekDay = date.getUTCDay();
    const weekendDrag = weekDay === 0 ? -2 : weekDay === 6 ? -1 : 0;
    const month = date.getUTCMonth();
    const dayOfMonth = date.getUTCDate();
    const planningLull = [7, 36, 67, 104, 137, 182, 226, 272, 319].includes(dayIndex - repoIndex);
    const incidentLull = (dayIndex >= 82 + repoIndex && dayIndex <= 84 + repoIndex)
      || (dayIndex >= 249 - repoIndex && dayIndex <= 251 - repoIndex);
    const summerSlowdown = month === 7 && dayOfMonth >= 5 && dayOfMonth <= 23;
    const holidayFreeze = month === 11 && dayOfMonth >= 21;
    const releaseRush = (dayIndex >= 54 && dayIndex <= 60)
      || (dayIndex >= 117 + repoIndex && dayIndex <= 124 + repoIndex)
      || (dayIndex >= 180 - repoIndex && dayIndex <= 187 - repoIndex)
      || (dayIndex >= 263 + repoIndex && dayIndex <= 271 + repoIndex)
      || (dayIndex >= 327 - repoIndex && dayIndex <= 333 - repoIndex);
    const reviewPileup = [12, 28, 53, 73, 96, 119, 142, 156, 177, 203, 234, 258, 286, 312, 341, 356].includes((dayIndex + repoIndex * 3) % 365);
    const base = 1 + ((dayIndex * 7 + repoIndex * 5) % 3);
    const churn = ((dayIndex * dayIndex + repoIndex * 11) % 7) === 0 ? 2 : 0;
    const maintenanceBurst = dayOfMonth === 1 && weekDay >= 1 && weekDay <= 4 ? 2 : 0;
    const count = Math.max(
      0,
      base
        + weekendDrag
        + churn
        + maintenanceBurst
        + (releaseRush ? 5 : 0)
        + (reviewPileup ? 4 : 0)
        - (planningLull ? 2 : 0)
        - (incidentLull ? 4 : 0)
        - (summerSlowdown ? 2 : 0)
        - (holidayFreeze ? 2 : 0),
    );

    return {
      dayIndex,
      commit_count: count,
      additions: count * (18 + ((dayIndex * 13 + repoIndex * 17) % 110)) + (releaseRush ? 260 + repoIndex * 35 : 0),
      deletions: count * (7 + ((dayIndex * 11 + repoIndex * 19) % 74)) + (incidentLull ? 40 : 0),
      contributor_count: count === 0 ? 0 : Math.min(users.length, 1 + ((dayIndex + repoIndex + (releaseRush || reviewPileup ? 2 : 0)) % users.length)),
    };
  });
}

function buildCommits(repo, repoIndex) {
  const dailyActivity = buildDailyActivity(repoIndex);
  const commits = [];

  dailyActivity.forEach((day, dayPosition) => {
    const intraDaySpike = day.commit_count >= 7 ? 1 : 0;
    for (let commitOfDay = 0; commitOfDay < day.commit_count; commitOfDay += 1) {
      const index = commits.length;
      const author = pick(users, dayPosition + commitOfDay + repoIndex + intraDaySpike);
      const additions = 12 + ((day.additions + index * 23 + commitOfDay * 47) % 690);
      const deletions = 3 + ((day.deletions + index * 17 + commitOfDay * 29) % 320);
      const hash = hashFor(repo.id, index);
      const parent = index > 0 ? hashFor(repo.id, index - 1) : null;
      const fileCount = Math.min(filePools[repo.id].length, 1 + ((dayPosition + commitOfDay + repoIndex) % 5) + (day.commit_count > 6 && commitOfDay % 3 === 0 ? 2 : 0));
      const fileOffset = (dayPosition * 2 + commitOfDay * 3 + repoIndex) % filePools[repo.id].length;
      const files = Array.from({ length: fileCount }, (_, fileIndex) => (
        filePools[repo.id][(fileOffset + fileIndex * 2) % filePools[repo.id].length]
      ));

      commits.push({
        hash,
        parent_hash: parent,
        message: pick(commitMessages, dayPosition + commitOfDay * 2 + repoIndex),
        author,
        additions,
        deletions,
        changed_files: [...new Set(files)],
        created_at: dateFromStart(day.dayIndex, 7 + ((commitOfDay * 2 + dayPosition + repoIndex) % 12), (commitOfDay * 11 + dayPosition * 3) % 60),
      });
    }
  });

  return commits.map((commit, index) => ({
    ...commit,
    parent_hash: index > 0 ? commits[index - 1].hash : null,
  })).sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
}

function buildBranches(repo, repoIndex, commits) {
  const names = branchNamesByRepo[repo.id];
  return names.map((name, index) => {
    const isDefault = name === repo.default_branch;
    const headIndex = Math.min(commits.length - 1, index * 9 + (isDefault ? 0 : 5));
    const head = commits[headIndex];
    const behind = isDefault ? 0 : 1 + ((index + repoIndex) % 8);
    const ahead = isDefault ? 0 : 2 + ((index * 3 + repoIndex) % 14);
    const staleDays = isDefault ? 1 : 3 + index * 8 + repoIndex;
    const status = isDefault ? "default" : staleDays < 15 ? "active" : staleDays <= 30 ? "idle" : "outdated";
    return {
      id: `${repo.id}:${name}`,
      repo_id: repo.id,
      name,
      last_commit_hash: head.hash,
      head_commit_hash: head.hash,
      merge_base_hash: commits[Math.min(commits.length - 1, headIndex + behind)]?.hash || head.hash,
      latest_commit: head,
      latestCommit: head,
      created_at: dateFromStart(Math.min(developmentDays - 1, 2 + index * 11 + repoIndex * 3), 9 + (index % 5)),
      last_activity_at: isoDaysAgo(staleDays),
      last_analyzed_at: isoDaysAgo(Math.max(0, staleDays - 1)),
      is_default: isDefault,
      isDefault,
      freshness_status: status,
      status,
      stale_days: staleDays,
      ahead_count: ahead,
      behind_count: behind,
      divergence_distance: ahead + behind,
      commit_count: Math.max(6, Math.round(commits.length / (index + 1))),
      activity_score: Math.max(12, 96 - staleDays * 2),
      activity_heat: Math.max(0.12, 1 - staleDays / 60),
      lane_color: ["#a5d6ff", "#58a6ff", "#ffba42", "#7ee787", "#d2a8ff", "#ff7b72"][index % 6],
    };
  });
}

function buildTimeline(repoIndex) {
  return buildDailyActivity(repoIndex).map((day) => ({
    bucket_start: dateFromStart(day.dayIndex, 0),
    commit_count: day.commit_count,
    additions: day.additions,
    deletions: day.deletions,
    contributor_count: day.contributor_count,
  }));
}

function repoOrThrow(repoId) {
  const repo = repositories.find((item) => item.id === repoId);
  if (!repo) {
    throw new Error("Demo repository not found");
  }
  return repo;
}

function delay(value, ms = 160) {
  return new Promise((resolve) => window.setTimeout(() => resolve(structuredClone(value)), ms));
}

function readonlyAction(name) {
  return Promise.reject(new Error(`${name} is disabled in the isolated demo`));
}

export async function fetchWithClerkAuth() {
  throw new Error("Network API calls are disabled in the isolated demo");
}

export async function fetchRepositories() {
  return delay(repositories);
}

export async function fetchBranches(repoId) {
  repoOrThrow(repoId);
  return delay(branchesByRepo[repoId]);
}

export async function fetchSystemHealth() {
  return delay({
    overall_status: "healthy",
    uptime_ms: 1000 * 60 * 60 * 21 + 1000 * 60 * 17,
    services: [
      { service: "frontend-demo", health: "healthy", status: "static", last_message: "Served from local mock data" },
      { service: "api-simulator", health: "healthy", status: "read-only", last_message: "All backend calls intercepted" },
      { service: "sync-monitor", health: "warning", status: "simulated", last_message: "Remote operations are disabled" },
      { service: "analytics", health: "healthy", status: "precomputed", last_message: "Full-year 2026 activity window loaded" },
    ],
    recent_logs: [
      { event: "demo-ready", message: "Isolated demo runtime initialized", timestamp: now.toISOString() },
      { event: "mutation-guard", message: "Create, update, delete, sync, and clone operations disabled", timestamp: isoDaysAgo(0, 9) },
    ],
  });
}

export async function fetchAnalyticsOverview(repoId) {
  repoOrThrow(repoId);
  const commits = commitsByRepo[repoId];
  const branches = branchesByRepo[repoId];
  const total = branches.reduce((sum, branch) => sum + branch.commit_count, 0);
  return delay({
    branches_count: branches.length,
    commits_count: commits.length,
    contributors_count: users.length,
    last_push_at: commits[0]?.created_at,
    last_pull_at: isoDaysAgo(1, 15),
    repository_size_bytes: 8200000 + commits.length * 46500,
    object_count: commits.length * 5 + branches.length * 3,
    branch_commit_distribution: branches.map((branch) => ({
      branch: branch.name,
      total_count: branch.commit_count,
      percentage: total ? (branch.commit_count / total) * 100 : 0,
    })),
  });
}

export async function fetchVcsAnalytics(repoId) {
  repoOrThrow(repoId);
  const timeline = timelinesByRepo[repoId];
  const totalChanges = filePools[repoId].map((path, index) => ({
    path,
    percentage: Math.max(8, 38 - index * 4 + (repoId.length % 5)),
  }));
  return delay({
    timeline,
    top_modified_files: totalChanges,
    branches: branchesByRepo[repoId],
    dag_metrics: {
      commits_count: commitsByRepo[repoId].length,
      total_commits: commitsByRepo[repoId].length,
      open_pull_requests: 4 + (repoId.length % 4),
      dag_complexity: 0.68,
    },
    topology_cache: [],
  });
}

export async function fetchBranchAnalytics(repoId) {
  return fetchVcsAnalytics(repoId);
}

export async function fetchActivityFeed(repoId, _getToken, limit = 10, action) {
  repoOrThrow(repoId);
  const commitItems = commitsByRepo[repoId].map((commit) => ({
    id: commit.hash,
    type: "commit",
    action: "commit",
    message: commit.message,
    actor: commit.author,
    branch_name: pick(branchNamesByRepo[repoId], commit.hash.charCodeAt(0)),
    commit_hash: commit.hash,
    created_at: commit.created_at,
  }));
  const syncItems = buildSyncLogs(repoId).map((log) => ({
    ...log,
    type: log.action,
    actor: pick(users, log.id.length),
  }));
  const items = [...commitItems, ...syncItems]
    .filter((item) => !action || item.action === action)
    .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
    .slice(0, limit);
  return delay({ items, limit, offset: 0, total: items.length });
}

export async function fetchSyncMonitor(repoId) {
  repoOrThrow(repoId);
  const logs = buildSyncLogs(repoId);
  return delay({
    logs,
    anomalies: [
      { level: "warn", event_type: "merge", branch_name: branchNamesByRepo[repoId][2], message: "Merge queue latency above rolling median", created_at: isoDaysAgo(3, 14) },
      { level: "info", event_type: "sync", branch_name: branchNamesByRepo[repoId][0], message: "Remote branch graph rebuilt from mock snapshot", created_at: isoDaysAgo(6, 11) },
      { level: "critical", event_type: "branch_delete", branch_name: "archive/prototype", message: "Deleted branch retained in audit trail", created_at: isoDaysAgo(14, 16) },
    ],
    failure_propagation: Array.from({ length: 8 }, (_, index) => ({
      bucket_start: isoDaysAgo((7 - index) * 3, 0),
      total_count: 2 + ((index + repoId.length) % 7),
      critical_count: index % 3 === 0 ? 1 : 0,
      warn_count: 1 + (index % 4),
    })),
    action_counts: {
      push_count: logs.filter((log) => log.action === "push").length,
      pull_count: logs.filter((log) => log.action === "pull").length,
      merge_count: logs.filter((log) => log.action === "merge").length,
      sync_count: logs.filter((log) => log.action === "sync" || log.action === "sync-db").length,
    },
  });
}

function buildSyncLogs(repoId) {
  const actions = ["push", "pull", "merge", "sync", "push", "pull", "sync-db", "merge", "push", "sync"];
  return actions.map((action, index) => {
    const branch = pick(branchNamesByRepo[repoId], index);
    const commit = commitsByRepo[repoId][index * 2] || commitsByRepo[repoId][0];
    return {
      id: `${repoId}-sync-${index}`,
      source: "demo",
      action,
      branch_name: branch,
      commit_hash: commit.hash,
      severity: index % 7 === 0 ? "critical" : index % 4 === 0 ? "warn" : "info",
      message: `${action} action recorded for ${branch}`,
      created_at: isoDaysAgo(index * 2, 9 + index),
    };
  });
}

export async function fetchCommitGraph(repoId, refName, _getToken, limit = 20) {
  repoOrThrow(repoId);
  const branch = branchesByRepo[repoId].find((item) => item.name === refName) || branchesByRepo[repoId][0];
  const nodes = commitsByRepo[repoId].slice(0, limit).map((commit, index) => ({
    hash: commit.hash,
    parent_hash: index < limit - 1 ? commitsByRepo[repoId][index + 1]?.hash || commit.parent_hash : commit.parent_hash,
    message: commit.message,
    author: commit.author,
    created_at: commit.created_at,
    branches: index === 0 ? [branch.name] : index % 9 === 0 ? [pick(branchNamesByRepo[repoId], index)] : [],
  }));
  return delay({
    repo_id: repoId,
    ref: branch.name,
    head: branch.last_commit_hash,
    nodes,
  });
}

export async function fetchCommitHistory(repoId, refName, _getToken, limit = 6) {
  repoOrThrow(repoId);
  const branch = branchesByRepo[repoId].find((item) => item.name === refName) || branchesByRepo[repoId][0];
  return delay({
    items: commitsByRepo[repoId].slice(0, limit).map((commit) => ({ ...commit, branch_name: branch.name })),
    limit,
    offset: 0,
    total: commitsByRepo[repoId].length,
  });
}

export async function initRepository() {
  return readonlyAction("Repository creation");
}

export async function deleteRepository() {
  return readonlyAction("Repository deletion");
}

export async function deleteAccountRecords() {
  return readonlyAction("Account deletion");
}

export async function updateAccountProfile() {
  return { ok: true, readonly: true };
}

export async function cloneRepositoryToDesktop() {
  return readonlyAction("Desktop clone");
}

export async function forceRecloneRepositoryToDesktop() {
  return readonlyAction("Desktop overwrite");
}
