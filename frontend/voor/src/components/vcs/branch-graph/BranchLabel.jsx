function truncateText(value, maxLength) {
  const text = String(value || "");
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 1))}...`;
}

export function BranchLabel({ label }) {
  const width = label.isDefault ? 124 : 164;
  const height = label.isDefault ? 42 : 56;
  const branchName = truncateText(label.branchName, label.isDefault ? 18 : 22);
  const statusText = label.isDefault
    ? "default branch"
    : `${label.ahead ?? 0} ahead / ${label.behind ?? 0} behind`;
  const hasHealth = !label.isDefault && Number.isFinite(Number(label.health));

  return (
    <g
      className={`branch-svg-label ${label.isDefault ? "default" : ""}`}
      style={{ "--branch-color": label.color || "#58a6ff" }}
      transform={`translate(${label.x} ${label.y})`}
    >
      <rect className="branch-svg-label-card" width={width} height={height} rx="5" />
      <circle className="branch-svg-label-dot" cx="13" cy="16" r="4" />
      <text className="branch-svg-label-name" x="24" y="19">
        {branchName}
      </text>
      <text className="branch-svg-label-meta" x="12" y={label.isDefault ? 34 : 36}>
        {statusText}
      </text>
      {hasHealth ? (
        <text className="branch-svg-label-health" x="12" y="50">
          health {Math.round(Number(label.health))}
        </text>
      ) : null}
    </g>
  );
}
