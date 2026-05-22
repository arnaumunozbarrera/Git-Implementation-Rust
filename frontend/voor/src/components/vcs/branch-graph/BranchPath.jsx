export function BranchPath({ onHover, path }) {
  const branchName = path.branchName || path.id;
  const classes = [
    "branch-path-layer",
    path.active ? "active" : "",
    path.muted ? "muted" : "",
    path.isDefault ? "default" : "",
    path.isMerge ? "merge" : "",
    path.isFork ? "fork" : "",
    `status-${path.status || "unknown"}`,
    `severity-${path.severity || "normal"}`,
  ].filter(Boolean).join(" ");
  const label = path.isDefault
    ? `${branchName} default branch timeline`
    : `${branchName}: ${path.ahead ?? 0} ahead, ${path.behind ?? 0} behind`;

  return (
    <g
      aria-label={label}
      className={classes}
      onBlur={() => onHover?.(null)}
      onFocus={() => onHover?.(branchName)}
      onMouseEnter={() => onHover?.(branchName)}
      onMouseLeave={() => onHover?.(null)}
      role="button"
      tabIndex="0"
    >
      <path className="branch-path-hitbox" d={path.d} fill="none" stroke="transparent" strokeLinecap="round" strokeLinejoin="round" />
      <path className="branch-path-shadow" d={path.d} fill="none" strokeLinecap="round" strokeLinejoin="round" />
      <path className="branch-path" d={path.d} fill="none" stroke={`url(#gradient-${cssEscape(path.id)})`} strokeLinecap="round" strokeLinejoin="round" />
    </g>
  );
}

function cssEscape(value) {
  return String(value).replace(/[^a-zA-Z0-9_-]/g, "-");
}
