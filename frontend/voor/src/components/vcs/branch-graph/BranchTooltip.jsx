function shortHash(value) {
  return String(value || "").slice(0, 10);
}

function formatTime(value) {
  if (!value) {
    return "";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

export function BranchTooltip({ node, width = 1040, height = 430 }) {
  if (!node) {
    return null;
  }

  const author = node.author?.username || node.author?.email || "";
  const time = formatTime(node.created_at || node.createdAt);
  const meta = [author, time].filter(Boolean).join(" / ");
  const left = clamp((Number(node.x) / Number(width || 1040)) * 100, 3, 82);
  const top = clamp((Number(node.y) / Number(height || 430)) * 100, 8, 92);
  const branchName = node.branchNames?.join(", ") || node.branchName || "";

  return (
    <div
      className="branch-graph-tooltip"
      style={{
        left: `${left}%`,
        top: `${top}%`,
      }}
    >
      <strong>{shortHash(node.hash) || "commit"}</strong>
      <span>{node.message || "Commit node"}</span>
      {meta ? <small>{meta}</small> : null}
      {branchName ? <code>{branchName}</code> : null}
    </div>
  );
}
