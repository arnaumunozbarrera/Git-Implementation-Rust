export function BranchMetrics({ metrics }) {
  return (
    <footer className="branch-metrics-row" aria-label="Repository topology metrics">
      {metrics.map((metric) => (
        <div className="branch-metric" key={metric.label} style={{ "--metric-accent": metric.accent }}>
          <span>{metric.label}</span>
          <strong>{metric.value}</strong>
        </div>
      ))}
    </footer>
  );
}
