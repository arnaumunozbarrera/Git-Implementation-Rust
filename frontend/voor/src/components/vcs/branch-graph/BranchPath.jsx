export function BranchPath({ path }) {
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

  return (
    <g className={classes}>
      <path className="branch-path-shadow" d={path.d} fill="none" strokeLinecap="round" strokeLinejoin="round" />
      <path className="branch-path" d={path.d} fill="none" stroke={`url(#gradient-${cssEscape(path.id)})`} strokeLinecap="round" strokeLinejoin="round" />
    </g>
  );
}

function cssEscape(value) {
  return String(value).replace(/[^a-zA-Z0-9_-]/g, "-");
}
