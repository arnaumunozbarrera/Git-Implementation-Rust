export function BranchNode({ active, hovered, node, onHover }) {
  const nodeClasses = [
    "commit-graph-node",
    node.isHead ? "head" : "",
    node.isDefault ? "default" : "",
    node.state === "stale" || node.state === "outdated" ? "stale" : "",
    hovered ? "hovered" : "",
    active ? "active" : "",
  ].filter(Boolean).join(" ");
  const label = [node.hash, node.message].filter(Boolean).join(" - ") || "Commit node";

  return (
    <g
      aria-label={label}
      className={nodeClasses}
      onBlur={() => onHover?.(null)}
      onFocus={() => onHover?.(node)}
      onMouseEnter={() => onHover?.(node)}
      onMouseLeave={() => onHover?.(null)}
      role="button"
      tabIndex="0"
      transform={`translate(${node.x} ${node.y})`}
    >
      {node.isHead ? <circle className="commit-head-pulse" r="17" /> : null}
      <circle className="commit-node-glow" r={node.isHead ? 15 : 12} />
      <circle className="commit-node-core" r={node.isHead ? 7 : 5.2} />
      {node.isHead ? (
        <g className="commit-head-label" transform="translate(13 -20)">
          <rect x="0" y="0" width="45" height="19" rx="4" />
          <text x="9" y="13">HEAD</text>
        </g>
      ) : null}
    </g>
  );
}
