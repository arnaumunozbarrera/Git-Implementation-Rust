const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? "/api";

export async function fetchWithClerkAuth(path, getToken, options = {}) {
  const token = await getToken();
  const response = await fetch(`${apiBaseUrl}${path}`, {
    ...options,
    headers: {
      Accept: "application/json",
      ...(options.body ? { "Content-Type": "application/json" } : {}),
      ...(options.headers ?? {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `API request failed with ${response.status}`);
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
