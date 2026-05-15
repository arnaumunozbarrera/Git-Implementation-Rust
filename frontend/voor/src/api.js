const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? "/api";
const authToken = import.meta.env.VITE_AUTH_TOKEN;

export async function fetchAnalyticsOverview(repoId) {
  const response = await fetch(`${apiBaseUrl}/repos/${encodeURIComponent(repoId)}/analytics/overview`, {
    headers: {
      Accept: "application/json",
      ...(authToken ? { Authorization: `Bearer ${authToken}` } : {}),
    },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Analytics request failed with ${response.status}`);
  }

  return response.json();
}
