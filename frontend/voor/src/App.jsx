const navItems = [
  { icon: "dashboard", label: "Overview", active: true },
  { icon: "analytics", label: "Activity" },
  { icon: "call_split", label: "Branches" },
  { icon: "sync", label: "Sync Monitor" },
];

export function App() {
  return (
    <div className="app-shell">
      <aside className="side-nav" aria-label="Primary">
        <div className="brand-block">
          <span className="material-symbols-outlined brand-icon">terminal</span>
          <div>
            <div className="brand-title">Git Voor</div>
          </div>
        </div>

        <nav className="nav-list">
          {navItems.map((item) => (
            <a className={`nav-item ${item.active ? "active" : ""}`} href="#" key={item.label}>
              <span className="material-symbols-outlined">{item.icon}</span>
              <span>{item.label}</span>
            </a>
          ))}
        </nav>

        <div className="nav-footer">
          <a className="nav-item" href="#">
            <span className="material-symbols-outlined">settings</span>
            <span>Settings</span>
          </a>
        </div>
      </aside>

      <header className="top-bar">
        <div className="repo-context">
          <div>
            <span className="repo-name">main-repo-v2</span>
            <span className="visibility-pill">Public</span>
          </div>
          <span className="sync-meta">
            <span className="material-symbols-outlined">history</span>
            Synced 2m ago
          </span>
        </div>
        <div className="top-actions">
          <div className="avatar" aria-label="User profile">VA</div>
        </div>
      </header>

      <main className="main-canvas">
        <section className="landing-heading">
          
        </section>

        <div className="single-component-layout" />
      </main>
    </div>
  );
}
