const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? "/api";

export async function fetchWithClerkAuth(path, getToken, options = {}) {
  const token = await getToken();
  if (!token) {
    throw new Error("Clerk did not return an auth token");
  }

  const response = await fetch(`${apiBaseUrl}${path}`, {
    ...options,
    headers: {
      Accept: "application/json",
      ...(options.body ? { "Content-Type": "application/json" } : {}),
      ...(options.headers ?? {}),
      Authorization: `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `API request failed with ${response.status}`);
  }

  return response.json();
}

export async function deleteRepository(repoId, getToken) {
  return fetchWithClerkAuth(`/repos/${encodeURIComponent(repoId)}`, getToken, {
    method: "DELETE",
  });
}

export async function deleteAccountRecords(getToken) {
  return fetchWithClerkAuth("/account", getToken, {
    method: "DELETE",
  });
}

export async function updateAccountProfile(profile, getToken) {
  return fetchWithClerkAuth("/account/profile", getToken, {
    method: "POST",
    body: JSON.stringify(profile),
  });
}

export async function fetchRepositories(getToken) {
  return fetchWithClerkAuth("/repos", getToken);
}

export async function initRepository(repository, getToken) {
  return fetchWithClerkAuth("/repos/init", getToken, {
    method: "POST",
    body: JSON.stringify(repository),
  });
}

export async function cloneRepositoryToDesktop(repoId, payload, getToken) {
  return fetchWithClerkAuth(`/repos/${encodeURIComponent(repoId)}/clone-desktop`, getToken, {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export async function fetchBranches(repoId, getToken) {
  return fetchWithClerkAuth(`/repos/${encodeURIComponent(repoId)}/branches`, getToken);
}

export async function fetchSystemHealth() {
  const response = await fetch(`${apiBaseUrl}/health`, {
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Health request failed with ${response.status}`);
  }

  return response.json();
}

export async function fetchAnalyticsOverview(repoId, getToken) {
  if (getToken) {
    return fetchWithClerkAuth(`/repos/${encodeURIComponent(repoId)}/analytics/overview`, getToken);
  }

  const response = await fetch(`${apiBaseUrl}/repos/${encodeURIComponent(repoId)}/analytics/overview`, {
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Analytics request failed with ${response.status}`);
  }

  return response.json();
}

export async function fetchVcsAnalytics(repoId, getToken) {
  return fetchWithClerkAuth(`/repos/${encodeURIComponent(repoId)}/analytics/vcs`, getToken);
}

export async function fetchActivityFeed(repoId, getToken, limit = 10, action) {
  const params = new URLSearchParams({ limit: String(limit), offset: "0" });
  if (action) {
    params.set("action", action);
  }

  return fetchWithClerkAuth(
    `/repos/${encodeURIComponent(repoId)}/activity?${params.toString()}`,
    getToken,
  );
}

export async function fetchCommitGraph(repoId, refName, getToken, limit = 20) {
  const params = new URLSearchParams({ limit: String(limit) });
  if (refName) {
    params.set("ref", refName);
  }

  return fetchWithClerkAuth(
    `/repos/${encodeURIComponent(repoId)}/commits/graph?${params.toString()}`,
    getToken,
  );
}

export async function fetchCommitHistory(repoId, refName, getToken, limit = 6) {
  const params = new URLSearchParams({ limit: String(limit), offset: "0" });
  if (refName) {
    params.set("ref", refName);
  }

  return fetchWithClerkAuth(
    `/repos/${encodeURIComponent(repoId)}/commits?${params.toString()}`,
    getToken,
  );
}
