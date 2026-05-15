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
  apiBaseUrl: "/api",
  authToken: "",
  activeRepoId: "main-repo-v2",
  defaultBranch: "main",
  defaultHead: "",
  repoName: "main-repo-v2",
  ownerId: "",
  repoVisibility: "public",
  repoDescription: "",
  readmePath: "README.md",
  tags: "analytics,vcs",
  syncAction: "pull",
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
  const [settingsTab, setSettingsTab] = useState("connection");
  const [settings, setSettings] = useState(readSettings);

  const updateSetting = (key, value) => {
    setSettings((current) => ({ ...current, [key]: value }));
  };

  const saveSettings = () => {
    localStorage.setItem("gitVoorSettings", JSON.stringify(settings));
  };

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
          <div className="avatar" aria-label="User profile">VA</div>
        </div>
      </header>

      <main className="main-canvas">
        {activePage === "settings" ? (
          <SettingsPage
            activeTab={settingsTab}
            onSave={saveSettings}
            onTabChange={setSettingsTab}
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

function SettingsPage({ activeTab, onSave, onTabChange, onUpdate, settings }) {
  const tabs = [
    { id: "connection", label: "Connection" },
    { id: "repository", label: "Repository" },
    { id: "sync", label: "Sync" },
  ];

  return (
    <section className="settings-page">
      <div className="landing-heading">
        <p className="label-caps">User Configuration</p>
        <h1>Settings</h1>
        <p>Basic defaults for the current backend routes and protected API workflows.</p>
      </div>

      <div className="settings-tabs" role="tablist" aria-label="Settings sections">
        {tabs.map((tab) => (
          <button
            className={`settings-tab ${activeTab === tab.id ? "active" : ""}`}
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            role="tab"
            type="button"
          >
            {tab.label}
          </button>
        ))}
      </div>

      {activeTab === "connection" ? (
        <SettingsPanel endpoint="GET /repos, GET /users, GET /repos/:repo_id/*" title="API Connection">
          <TextField label="API base URL" value={settings.apiBaseUrl} onChange={(value) => onUpdate("apiBaseUrl", value)} />
          <TextField label="Bearer token" type="password" value={settings.authToken} onChange={(value) => onUpdate("authToken", value)} />
          <TextField label="Active repo id" value={settings.activeRepoId} onChange={(value) => onUpdate("activeRepoId", value)} />
        </SettingsPanel>
      ) : null}

      {activeTab === "repository" ? (
        <SettingsPanel endpoint="POST /repos/init" title="Repository Defaults">
          <div className="form-grid">
            <TextField label="Repository id" value={settings.activeRepoId} onChange={(value) => onUpdate("activeRepoId", value)} />
            <TextField label="Repository name" value={settings.repoName} onChange={(value) => onUpdate("repoName", value)} />
            <TextField label="Owner id" value={settings.ownerId} onChange={(value) => onUpdate("ownerId", value)} />
            <TextField label="Default branch" value={settings.defaultBranch} onChange={(value) => onUpdate("defaultBranch", value)} />
            <TextField label="Readme path" value={settings.readmePath} onChange={(value) => onUpdate("readmePath", value)} />
            <TextField label="Tags" value={settings.tags} onChange={(value) => onUpdate("tags", value)} />
          </div>
          <label className="field-label">
            Visibility
            <select value={settings.repoVisibility} onChange={(event) => onUpdate("repoVisibility", event.target.value)}>
              <option value="public">Public</option>
              <option value="private">Private</option>
            </select>
          </label>
          <label className="field-label">
            Description
            <textarea value={settings.repoDescription} onChange={(event) => onUpdate("repoDescription", event.target.value)} rows="3" />
          </label>
        </SettingsPanel>
      ) : null}

      {activeTab === "sync" ? (
        <SettingsPanel endpoint="POST /push, POST /pull, POST /sync-db" title="Sync Defaults">
          <div className="form-grid">
            <TextField label="Repository id" value={settings.activeRepoId} onChange={(value) => onUpdate("activeRepoId", value)} />
            <TextField label="Branch" value={settings.defaultBranch} onChange={(value) => onUpdate("defaultBranch", value)} />
            <TextField label="Head commit" value={settings.defaultHead} onChange={(value) => onUpdate("defaultHead", value)} />
          </div>
          <label className="field-label">
            Default action
            <select value={settings.syncAction} onChange={(event) => onUpdate("syncAction", event.target.value)}>
              <option value="pull">Pull</option>
              <option value="push">Push</option>
              <option value="sync-db">Sync DB</option>
            </select>
          </label>
        </SettingsPanel>
      ) : null}

      <div className="settings-actions">
        <button className="secondary-button" type="button" onClick={onSave}>Save Settings</button>
      </div>
    </section>
  );
}

function SettingsPanel({ children, endpoint, title }) {
  return (
    <section className="settings-panel">
      <header className="settings-panel-header">
        <h2>{title}</h2>
        <span>{endpoint}</span>
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
