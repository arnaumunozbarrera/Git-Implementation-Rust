import { useState } from "react";

const navItems = [
  { id: "overview", icon: "dashboard" },
  { id: "activity", icon: "analytics" },
  { id: "branches", icon: "call_split" },
  { id: "sync", icon: "sync" },
];

const translations = {
  en: {
    account: {
      changeAccount: "Change account",
      logout: "Logout",
      profileLabel: "User profile",
    },
    appearance: {
      dark: "Dark mode",
      light: "Light mode",
    },
    nav: {
      overview: "Overview",
      activity: "Activity",
      branches: "Branches",
      sync: "Sync Monitor",
      settings: "Settings",
    },
    pages: {
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
    },
    settings: {
      eyebrow: "User Configuration",
      title: "Settings",
      description: "Application language, visual mode, and profile personalization.",
      interfaceEyebrow: "Interface",
      preferencesTitle: "Application Preferences",
      language: "Language",
      english: "English",
      spanish: "Spanish",
      appearance: "Appearance",
      profileEyebrow: "Profile",
      profileTitle: "User Personalization",
      displayName: "Display name",
      username: "Username",
      email: "Email",
      initials: "Profile initials",
      save: "Save Settings",
    },
  },
  es: {
    account: {
      changeAccount: "Cambiar cuenta",
      logout: "Cerrar sesión",
      profileLabel: "Perfil de usuario",
    },
    appearance: {
      dark: "Modo oscuro",
      light: "Modo claro",
    },
    nav: {
      overview: "Resumen",
      activity: "Actividad",
      branches: "Ramas",
      sync: "Monitor de sincronización",
      settings: "Ajustes",
    },
    pages: {
      overview: {
        eyebrow: "Panel de control del repositorio",
        title: "Resumen",
        description: "Este espacio está reservado para los módulos de resumen del repositorio.",
      },
      activity: {
        eyebrow: "Eventos del repositorio",
        title: "Actividad",
        description: "Este espacio está reservado para commits y flujos de actividad de acceso.",
      },
      branches: {
        eyebrow: "Grafo de versiones",
        title: "Ramas",
        description: "Este espacio está reservado para el grafo de ramas y módulos de comparación.",
      },
      sync: {
        eyebrow: "Operaciones remotas",
        title: "Monitor de sincronización",
        description: "Este espacio está reservado para telemetría de push, pull y sincronización de base de datos.",
      },
    },
    settings: {
      eyebrow: "Configuración de usuario",
      title: "Ajustes",
      description: "Idioma de la aplicación, modo visual y personalización del perfil.",
      interfaceEyebrow: "Interfaz",
      preferencesTitle: "Preferencias de la aplicación",
      language: "Idioma",
      english: "Inglés",
      spanish: "Español",
      appearance: "Apariencia",
      profileEyebrow: "Perfil",
      profileTitle: "Personalización del usuario",
      displayName: "Nombre visible",
      username: "Usuario",
      email: "Correo electrónico",
      initials: "Iniciales del perfil",
      save: "Guardar ajustes",
    },
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
  const copy = translations[settings.language] ?? translations.en;

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
              <span>{copy.nav[item.id]}</span>
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
            <span>{copy.nav.settings}</span>
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
              aria-label={copy.account.profileLabel}
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
                <button type="button">{copy.account.changeAccount}</button>
                <button type="button">{copy.account.logout}</button>
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
            copy={copy}
            settings={settings}
          />
        ) : (
          <EmptySection page={copy.pages[activePage]} />
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

function SettingsPage({ copy, onSave, onUpdate, settings }) {
  return (
    <section className="settings-page">
      <div className="landing-heading">
        <p className="label-caps">{copy.settings.eyebrow}</p>
        <h1>{copy.settings.title}</h1>
        <p>{copy.settings.description}</p>
      </div>

      <div className="settings-stack">
        <SettingsPanel eyebrow={copy.settings.interfaceEyebrow} title={copy.settings.preferencesTitle}>
          <div className="form-grid">
            <label className="field-label">
              {copy.settings.language}
              <select value={settings.language} onChange={(event) => onUpdate("language", event.target.value)}>
                <option value="en">{copy.settings.english}</option>
                <option value="es">{copy.settings.spanish}</option>
              </select>
            </label>
            <label className="field-label">
              {copy.settings.appearance}
              <select value={settings.theme} onChange={(event) => onUpdate("theme", event.target.value)}>
                <option value="dark">{copy.appearance.dark}</option>
                <option value="light">{copy.appearance.light}</option>
              </select>
            </label>
          </div>
        </SettingsPanel>

        <SettingsPanel eyebrow={copy.settings.profileEyebrow} title={copy.settings.profileTitle}>
          <div className="form-grid">
            <TextField label={copy.settings.displayName} value={settings.displayName} onChange={(value) => onUpdate("displayName", value)} />
            <TextField label={copy.settings.username} value={settings.username} onChange={(value) => onUpdate("username", value)} />
            <TextField label={copy.settings.email} type="email" value={settings.email} onChange={(value) => onUpdate("email", value)} />
            <TextField label={copy.settings.initials} value={settings.initials} onChange={(value) => onUpdate("initials", value.slice(0, 3).toUpperCase())} />
          </div>
        </SettingsPanel>
      </div>

      <div className="settings-actions">
        <button className="secondary-button" type="button" onClick={onSave}>{copy.settings.save}</button>
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
