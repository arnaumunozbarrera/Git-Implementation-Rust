function metricAccent(metric) {
  if (metric?.accent) {
    return metric.accent;
  }

  const label = String(metric?.label || "").toLowerCase();
  if (label.includes("stale") || label.includes("risk")) {
    return "#ffba42";
  }
  if (label.includes("velocity") || label.includes("merge") || label.includes("health")) {
    return "#3fb950";
  }

  return "#e0e2ea";
}

export function BranchMetrics({ metrics }) {
  const items = Array.isArray(metrics) ? metrics.slice(0, 4) : [];

  if (!items.length) {
    return null;
  }

  return (
    <footer className="branch-metrics-row" aria-label="Repository topology metrics">
      {items.map((metric) => (
        <div className="branch-metric" key={metric.label} style={{ "--metric-accent": metricAccent(metric) }}>
          <span>{metric.label}</span>
          <strong>{metric.value}</strong>
        </div>
      ))}
    </footer>
  );
}
