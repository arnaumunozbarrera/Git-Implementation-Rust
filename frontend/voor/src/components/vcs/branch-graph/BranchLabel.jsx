export function BranchLabel({ label }) {
  const width = label.isDefault ? 118 : 148;
  const height = label.isDefault ? 40 : 50;
  const statusText = label.isDefault
    ? "default branch"
    : `${label.ahead} ahead / ${label.behind} behind`;

  return (
    <g className={`branch-svg-label ${label.isDefault ? "default" : ""}`} transform={`translate(${label.x} ${label.y})`}>
      <rect className="branch-svg-label-card" width={width} height={height} rx="6" />
      <circle className="branch-svg-label-dot" cx="12" cy="15" r="4" style={{ color: label.color }} />
      <text className="branch-svg-label-name" x="23" y="18">
        {label.branchName}
      </text>
      <text className="branch-svg-label-meta" x="12" y={label.isDefault ? 32 : 34}>
        {statusText}
      </text>
      {!label.isDefault && Number.isFinite(Number(label.health)) ? (
        <text className="branch-svg-label-health" x="12" y="46">
          health {Math.round(Number(label.health))}
        </text>
      ) : null}
    </g>
  );
}
