export const branchStatusMeta = {
  active: {
    label: "ACTIVE",
    description: "Recently updated branch (<15d)",
    color: "#7ee787",
  },
  idle: {
    label: "IDLE",
    description: "No commits between 15-30d",
    color: "#ffba42",
  },
  outdated: {
    label: "OUTDATED",
    description: "No commits for more than 30d",
    color: "#ff7b72",
  },
  default: {
    label: "DEFAULT",
    description: "Repository primary branch",
    color: "#58a6ff",
  },
};

export function daysSince(value) {
  if (!value) {
    return Number.POSITIVE_INFINITY;
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return Number.POSITIVE_INFINITY;
  }

  return Math.max(0, Math.floor((Date.now() - date.getTime()) / 86400000));
}

export function detectBranchStatus(branch, defaultBranchName) {
  if (branch?.name === defaultBranchName) {
    return "default";
  }

  const age = daysSince(branch?.latestCommit?.created_at || branch?.created_at);
  if (age < 15) {
    return "active";
  }

  if (age <= 30) {
    return "idle";
  }

  return "outdated";
}
