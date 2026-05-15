import { useState } from "react";

const navItems = [
  { id: "overview", icon: "dashboard", label: "Overview" },
  { id: "activity", icon: "analytics", label: "Activity" },
  { id: "branches", icon: "call_split", label: "Branches" },
  { id: "sync", icon: "sync", label: "Sync Monitor" },
];

const emptyPages = {
  overview: {
    eyebrow: "Repository Control Plane",
    title: "Overview",
    description: "This workspace is reserved for repository summary modules.",
  },
  activity: {
    eyebrow: "Repository Events",
    title: "Activity",
    description: "This workspace is reserved for commit and access activity streams.",
  },
  branches: {
    eyebrow: "Version Graph",
    title: "Branches",
    description: "This workspace is reserved for branch graph and comparison modules.",
  },
  sync: {
    eyebrow: "Remote Operations",
    title: "Sync Monitor",
    description: "This workspace is reserved for push, pull, and database sync telemetry.",
  },
};

const settingsDefaults = {
  activeRepoId: "main-repo-v2",
  defaultBranch: "main",
  repoVisibility: "public",
  language: "en",
  theme: "dark",
  displayName: "Voor Admin",
  username: "voor_admin",
  email: "admin@gitvoor.local",
  initials: "VA",
};

function readSettings() {
  try {
    return { ...settingsDefaults, ...JSON.parse(localStorage.getItem("gitVoorSettings") ?? "{}") };
  } catch {
    return settingsDefaults;
  }
}

export function App() {
  const [activePage, setActivePage] = useState("overview");
  const [settings, setSettings] = useState(readSettings);
  const [accountMenuOpen, setAccountMenuOpen] = useState(false);

  const updateSetting = (key, value) => {
    setSettings((current) => ({ ...current, [key]: value }));
  };

  const saveSettings = () => {
    localStorage.setItem("gitVoorSettings", JSON.stringify(settings));
  };

  const appClassName = `app-shell theme-${settings.theme}`;
  const labels = settings.language === "es"
    ? {
        logout: "Cerrar sesion",
        changeAccount: "Cambiar cuenta",
      }
    : {
        logout: "Logout",
        changeAccount: "Change account",
      };

  return (
    <div className={appClassName}>
      <aside className="side-nav" aria-label="Primary">
        <div className="brand-block">
          <span className="material-symbols-outlined brand-icon">terminal</span>
          <div>
            <div className="brand-title">Git Voor</div>
          </div>
        </div>

        <nav className="nav-list">
          {navItems.map((item) => (
            <button
              className={`nav-item ${activePage === item.id ? "active" : ""}`}
              key={item.id}
              onClick={() => setActivePage(item.id)}
              type="button"
            >
              <span className="material-symbols-outlined">{item.icon}</span>
              <span>{item.label}</span>
            </button>
          ))}
        </nav>

        <div className="nav-footer">
          <button
            className={`nav-item ${activePage === "settings" ? "active" : ""}`}
            onClick={() => setActivePage("settings")}
            type="button"
          >
            <span className="material-symbols-outlined">settings</span>
            <span>Settings</span>
          </button>
        </div>
      </aside>

      <header className="top-bar">
        <div className="repo-context">
          <div>
            <span className="repo-name">{settings.activeRepoId}</span>
            <span className="visibility-pill">{settings.repoVisibility}</span>
          </div>
          <span className="sync-meta">
            <span className="material-symbols-outlined">history</span>
            {settings.defaultBranch}
          </span>
        </div>
        <div className="top-actions">
          <div className="account-menu">
            <button
              className="avatar"
              type="button"
              aria-expanded={accountMenuOpen}
              aria-label="User profile"
              onClick={() => setAccountMenuOpen((open) => !open)}
            >
              {settings.initials || "VA"}
            </button>
            {accountMenuOpen ? (
              <div className="account-popover">
                <div className="account-summary">
                  <strong>{settings.displayName}</strong>
                  <span>{settings.email}</span>
                </div>
                <button type="button">{labels.changeAccount}</button>
                <button type="button">{labels.logout}</button>
              </div>
            ) : null}
          </div>
        </div>
      </header>

      <main className="main-canvas">
        {activePage === "settings" ? (
          <SettingsPage
            onSave={saveSettings}
            onUpdate={updateSetting}
            settings={settings}
          />
        ) : (
          <EmptySection page={emptyPages[activePage]} />
        )}
      </main>
    </div>
  );
}

function EmptySection({ page }) {
  return (
    <section className="workspace-section">
      <div className="landing-heading">
        <p className="label-caps">{page.eyebrow}</p>
        <h1>{page.title}</h1>
        <p>{page.description}</p>
      </div>
      <div className="empty-workspace">
        <span className="material-symbols-outlined">space_dashboard</span>
      </div>
    </section>
  );
}

function SettingsPage({ onSave, onUpdate, settings }) {
  return (
    <section className="settings-page">
      <div className="landing-heading">
        <p className="label-caps">User Configuration</p>
        <h1>Settings</h1>
        <p>Application language, visual mode, and profile personalization.</p>
      </div>

      <div className="settings-stack">
        <SettingsPanel eyebrow="Interface" title="Application Preferences">
          <div className="form-grid">
            <label className="field-label">
              Language
              <select value={settings.language} onChange={(event) => onUpdate("language", event.target.value)}>
                <option value="en">English</option>
                <option value="es">Spanish</option>
              </select>
            </label>
            <label className="field-label">
              Appearance
              <select value={settings.theme} onChange={(event) => onUpdate("theme", event.target.value)}>
                <option value="dark">Dark mode</option>
                <option value="light">Light mode</option>
              </select>
            </label>
          </div>
        </SettingsPanel>

        <SettingsPanel eyebrow="Profile" title="User Personalization">
          <div className="form-grid">
            <TextField label="Display name" value={settings.displayName} onChange={(value) => onUpdate("displayName", value)} />
            <TextField label="Username" value={settings.username} onChange={(value) => onUpdate("username", value)} />
            <TextField label="Email" type="email" value={settings.email} onChange={(value) => onUpdate("email", value)} />
            <TextField label="Profile initials" value={settings.initials} onChange={(value) => onUpdate("initials", value.slice(0, 3).toUpperCase())} />
          </div>
        </SettingsPanel>
      </div>

      <div className="settings-actions">
        <button className="secondary-button" type="button" onClick={onSave}>Save Settings</button>
      </div>
    </section>
  );
}

function SettingsPanel({ children, eyebrow, title }) {
  return (
    <section className="settings-panel">
      <header className="settings-panel-header">
        <h2>{title}</h2>
        <span>{eyebrow}</span>
      </header>
      <div className="settings-panel-body">{children}</div>
    </section>
  );
}

function TextField({ label, onChange, type = "text", value }) {
  return (
    <label className="field-label">
      {label}
      <input type={type} value={value} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}
