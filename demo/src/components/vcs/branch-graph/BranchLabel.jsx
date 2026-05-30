function truncateText(value, maxLength) {
  const text = String(value || "");
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 1))}...`;
}

export function BranchLabel({ hovered, label, onHover }) {
  const branchName = truncateText(label.branchName, label.isDefault ? 18 : 24);
  const meta = label.isDefault
    ? "default"
    : `${label.ahead ?? 0} ahead / ${label.behind ?? 0} behind`;
  const width = Math.max(112, Math.min(188, branchName.length * 7.2 + meta.length * 3.4 + 34));

  return (
    <g
      aria-label={`${label.branchName} ${meta}`}
      className={`branch-svg-label ${label.isDefault ? "default" : ""} ${hovered ? "hovered" : ""}`}
      onBlur={() => onHover?.(null)}
      onFocus={() => onHover?.(label.branchName)}
      onMouseEnter={() => onHover?.(label.branchName)}
      onMouseLeave={() => onHover?.(null)}
      role="button"
      style={{ "--branch-color": label.color || "#58a6ff" }}
      tabIndex="0"
      transform={`translate(${label.x} ${label.y})`}
    >
      <rect className="branch-svg-label-pill" x="-8" y="-16" width={width} height="31" rx="15.5" />
      <circle className="branch-svg-label-dot" cx="5" cy="0" r="4" />
      <text className="branch-svg-label-name" x="15" y="-2">
        {branchName}
      </text>
      <text className="branch-svg-label-meta" x="15" y="10">
        {meta}
      </text>
    </g>
  );
}
