export function BranchNode({ active, hovered, node, onHover }) {
  const nodeClasses = [
    "commit-graph-node",
    node.isHead ? "head" : "",
    node.isDefault ? "default" : "",
    node.state === "stale" ? "stale" : "",
    hovered ? "hovered" : "",
    active ? "active" : "",
  ].filter(Boolean).join(" ");

  return (
    <g
      className={nodeClasses}
      onMouseEnter={() => onHover(node)}
      onMouseLeave={() => onHover(null)}
      tabIndex="0"
      transform={`translate(${node.x} ${node.y})`}
    >
      {node.isHead ? <circle className="commit-head-pulse" r="17" /> : null}
      <circle className="commit-node-glow" r={node.isHead ? 13 : 10} />
      <circle className="commit-node-core" r={node.isHead ? 6.5 : 4.8} />
      {node.isHead ? (
        <g className="commit-head-label" transform="translate(12 -18)">
          <rect x="0" y="0" width="39" height="18" rx="3" />
          <text x="8" y="12">HEAD</text>
        </g>
      ) : null}
    </g>
  );
}
