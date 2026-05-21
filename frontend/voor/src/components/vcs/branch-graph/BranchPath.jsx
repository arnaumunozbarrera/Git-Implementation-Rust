export function BranchPath({ path }) {
  return (
    <g className={`branch-path-layer ${path.active ? "active" : ""} ${path.muted ? "muted" : ""} ${path.isDefault ? "default" : ""} ${path.isMerge ? "merge" : ""} severity-${path.severity}`}>
      <path className="branch-path-shadow" d={path.d} />
      <path className="branch-path" d={path.d} stroke={`url(#gradient-${cssEscape(path.id)})`} />
    </g>
  );
}

function cssEscape(value) {
  return String(value).replace(/[^a-zA-Z0-9_-]/g, "-");
}
