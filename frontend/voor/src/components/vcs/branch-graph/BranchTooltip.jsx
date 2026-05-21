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

export function BranchTooltip({ node }) {
  if (!node) {
    return null;
  }

  const author = node.author?.username || node.author?.email || "";
  const time = formatTime(node.created_at);
  const meta = [author, time].filter(Boolean).join(" / ");

  return (
    <div
      className="branch-graph-tooltip"
      style={{
        left: `${(node.x / 1040) * 100}%`,
        top: `${(node.y / 430) * 100}%`,
      }}
    >
      <strong>{shortHash(node.hash)}</strong>
      <span>{node.message}</span>
      {meta ? <small>{meta}</small> : null}
      <code>{node.branchNames?.join(", ") || node.branchName}</code>
    </div>
  );
}
