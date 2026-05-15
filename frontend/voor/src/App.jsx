import { SignIn, SignedIn, SignedOut, useAuth, useClerk, useUser } from "@clerk/clerk-react";
import { useEffect, useState } from "react";
import { deleteAccountRecords, deleteRepository } from "./api.js";

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
    auth: {
      eyebrow: "Secure Workspace",
      title: "Sign in to Git Voor",
      description: "Authenticate with Clerk to access repository telemetry and protected backend routes.",
      missingTitle: "Clerk is not configured",
      missingDescription: "Set VITE_CLERK_PUBLISHABLE_KEY in the frontend environment to enable login.",
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
      save: "Save Settings",
      saved: "Settings saved",
      profileSaved: "Clerk profile updated",
      profileError: "Unable to update Clerk profile",
      emailLocked: "Primary email is managed by Clerk account settings.",
      dangerEyebrow: "Danger Zone",
      dangerTitle: "Repository and Account Removal",
      dangerDescription: "These actions permanently remove data from the Voor backend.",
      removeRepository: "Remove Repository",
      removeRepositoryHelp: "Deletes the active repository and its related backend records.",
      deleteAccount: "Delete Account",
      deleteAccountHelp: "Deletes your backend account records and then removes your Clerk account.",
      confirmRemoveRepository: "Remove this repository and its backend records?",
      confirmDeleteAccount: "Delete your account and backend records?",
      repositoryDeleted: "Repository removed",
      accountDeleted: "Account deleted",
      destructiveError: "Unable to complete destructive action",
    },
  },
  es: {
    account: {
      changeAccount: "Cambiar cuenta",
      logout: "Cerrar sesion",
      profileLabel: "Perfil de usuario",
    },
    appearance: {
      dark: "Modo oscuro",
      light: "Modo claro",
    },
    auth: {
      eyebrow: "Espacio seguro",
      title: "Inicia sesion en Git Voor",
      description: "Autenticate con Clerk para acceder a la telemetria del repositorio y rutas protegidas del backend.",
      missingTitle: "Clerk no esta configurado",
      missingDescription: "Define VITE_CLERK_PUBLISHABLE_KEY en el entorno del frontend para activar el inicio de sesion.",
    },
    nav: {
      overview: "Resumen",
      activity: "Actividad",
      branches: "Ramas",
      sync: "Monitor de sincronizacion",
      settings: "Ajustes",
    },
    pages: {
      overview: {
        eyebrow: "Panel de control del repositorio",
        title: "Resumen",
        description: "Este espacio esta reservado para los modulos de resumen del repositorio.",
      },
      activity: {
        eyebrow: "Eventos del repositorio",
        title: "Actividad",
        description: "Este espacio esta reservado para commits y flujos de actividad de acceso.",
      },
      branches: {
        eyebrow: "Grafo de versiones",
        title: "Ramas",
        description: "Este espacio esta reservado para el grafo de ramas y modulos de comparacion.",
      },
      sync: {
        eyebrow: "Operaciones remotas",
        title: "Monitor de sincronizacion",
        description: "Este espacio esta reservado para telemetria de push, pull y sincronizacion de base de datos.",
      },
    },
    settings: {
      eyebrow: "Configuracion de usuario",
      title: "Ajustes",
      description: "Idioma de la aplicacion, modo visual y personalizacion del perfil.",
      interfaceEyebrow: "Interfaz",
      preferencesTitle: "Preferencias de la aplicacion",
      language: "Idioma",
      english: "Ingles",
      spanish: "Espanol",
      appearance: "Apariencia",
      profileEyebrow: "Perfil",
      profileTitle: "Personalizacion del usuario",
      displayName: "Nombre visible",
      username: "Usuario",
      email: "Correo electronico",
      save: "Guardar ajustes",
      saved: "Ajustes guardados",
      profileSaved: "Perfil de Clerk actualizado",
      profileError: "No se pudo actualizar el perfil de Clerk",
      emailLocked: "El correo principal se gestiona desde los ajustes de cuenta de Clerk.",
      dangerEyebrow: "Zona de riesgo",
      dangerTitle: "Eliminar repositorio y cuenta",
      dangerDescription: "Estas acciones eliminan permanentemente datos del backend de Voor.",
      removeRepository: "Eliminar repositorio",
      removeRepositoryHelp: "Elimina el repositorio activo y sus registros relacionados del backend.",
      deleteAccount: "Eliminar cuenta",
      deleteAccountHelp: "Elimina tus registros del backend y despues elimina tu cuenta de Clerk.",
      confirmRemoveRepository: "Eliminar este repositorio y sus registros del backend?",
      confirmDeleteAccount: "Eliminar tu cuenta y registros del backend?",
      repositoryDeleted: "Repositorio eliminado",
      accountDeleted: "Cuenta eliminada",
      destructiveError: "No se pudo completar la accion destructiva",
    },
  },
};

const settingsDefaults = {
  activeRepoId: "main-repo-v2",
  defaultBranch: "main",
  repoVisibility: "public",
  language: "en",
  theme: "dark",
  displayName: "",
  username: "",
  email: "",
};

const clerkAppearance = {
  variables: {
    colorBackground: "#161b22",
    colorInputBackground: "#0d1117",
    colorInputText: "#e0e2ea",
    colorPrimary: "#58a6ff",
    colorText: "#e0e2ea",
    colorTextSecondary: "#c0c7d4",
    borderRadius: "4px",
    fontFamily: "Inter, sans-serif",
  },
  elements: {
    cardBox: "clerk-card-box",
    card: "clerk-card",
    headerTitle: "clerk-title",
    headerSubtitle: "clerk-subtitle",
    formButtonPrimary: "clerk-primary-button",
    formFieldInput: "clerk-input",
    footerActionLink: "clerk-link",
    socialButtonsBlockButton: "clerk-social-button",
  },
};

function readSettings() {
  try {
    const { initials, ...storedSettings } = JSON.parse(localStorage.getItem("gitVoorSettings") ?? "{}");
    return { ...settingsDefaults, ...storedSettings };
  } catch {
    return settingsDefaults;
  }
}

function initialsFromUsername(username) {
  const name = username || "VA";
  const parts = name
    .split(/[ ._@-]+/)
    .filter(Boolean)
    .map((part) => part[0]);

  return (parts.length > 1 ? parts.slice(0, 2).join("") : name.slice(0, 2))
    .toUpperCase();
}

export function App() {
  const [settings, setSettings] = useState(readSettings);
  const copy = translations[settings.language] ?? translations.en;

  return (
    <>
      <SignedOut>
        <LoginPage copy={copy} theme={settings.theme} />
      </SignedOut>
      <SignedIn>
        <AuthenticatedShell copy={copy} settings={settings} setSettings={setSettings} />
      </SignedIn>
    </>
  );
}

function AuthenticatedShell({ copy, settings, setSettings }) {
  const { openSignIn, signOut } = useClerk();
  const { getToken } = useAuth();
  const { user } = useUser();
  const [activePage, setActivePage] = useState("overview");
  const [accountMenuOpen, setAccountMenuOpen] = useState(false);
  const [saveStatus, setSaveStatus] = useState("");

  useEffect(() => {
    if (!user) {
      return;
    }

    setSettings((current) => ({
      ...current,
      displayName: current.displayName || user.fullName || "",
      email: current.email || user.primaryEmailAddress?.emailAddress || "",
      username: current.username || user.username || "",
    }));
  }, [setSettings, user]);

  const updateSetting = (key, value) => {
    setSaveStatus("");
    setSettings((current) => ({ ...current, [key]: value }));
  };

  const saveSettings = async () => {
    const { initials, ...settingsToSave } = settings;
    localStorage.setItem("gitVoorSettings", JSON.stringify(settingsToSave));

    try {
      if (user) {
        const [firstName, ...lastNameParts] = settings.displayName.trim().split(/\s+/);
        await user.update({
          firstName: firstName || undefined,
          lastName: lastNameParts.join(" ") || undefined,
          username: settings.username || undefined,
        });
      }
      setSaveStatus(copy.settings.profileSaved);
    } catch {
      setSaveStatus(copy.settings.profileError);
    }
  };

  const handleChangeAccount = async () => {
    setAccountMenuOpen(false);
    await signOut();
    openSignIn();
  };

  const handleLogout = async () => {
    setAccountMenuOpen(false);
    await signOut();
  };

  const handleDeleteRepository = async () => {
    if (!window.confirm(copy.settings.confirmRemoveRepository)) {
      return;
    }

    try {
      await deleteRepository(settings.activeRepoId, getToken);
      setSaveStatus(copy.settings.repositoryDeleted);
    } catch {
      setSaveStatus(copy.settings.destructiveError);
    }
  };

  const handleDeleteAccount = async () => {
    if (!window.confirm(copy.settings.confirmDeleteAccount)) {
      return;
    }

    try {
      await deleteAccountRecords(getToken);
      localStorage.removeItem("gitVoorSettings");
      await user?.delete();
      setSaveStatus(copy.settings.accountDeleted);
    } catch {
      setSaveStatus(copy.settings.destructiveError);
    }
  };

  const appClassName = `app-shell theme-${settings.theme}`;
  const profileInitials = initialsFromUsername(settings.username || user?.username || user?.primaryEmailAddress?.emailAddress);

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
              {profileInitials}
            </button>
            {accountMenuOpen ? (
              <div className="account-popover">
                <div className="account-summary">
                  <strong>{settings.displayName || user?.fullName || user?.username}</strong>
                  <span>{settings.email || user?.primaryEmailAddress?.emailAddress}</span>
                </div>
                <button type="button" onClick={handleChangeAccount}>{copy.account.changeAccount}</button>
                <button type="button" onClick={handleLogout}>{copy.account.logout}</button>
              </div>
            ) : null}
          </div>
        </div>
      </header>

      <main className="main-canvas">
        {activePage === "settings" ? (
          <SettingsPage
            copy={copy}
            onDeleteAccount={handleDeleteAccount}
            onDeleteRepository={handleDeleteRepository}
            onSave={saveSettings}
            onUpdate={updateSetting}
            saveStatus={saveStatus}
            settings={settings}
          />
        ) : (
          <EmptySection page={copy.pages[activePage]} />
        )}
      </main>
    </div>
  );
}

function LoginPage({ copy, theme }) {
  return (
    <main className={`login-page theme-${theme}`}>
      <section className="login-shell">
        <div className="login-copy">
          <span className="material-symbols-outlined brand-icon">terminal</span>
          <p className="label-caps">{copy.auth.eyebrow}</p>
          <h1>{copy.auth.title}</h1>
          <p>{copy.auth.description}</p>
        </div>
        <div className="login-form">
          <SignIn
            appearance={clerkAppearance}
            routing="hash"
            signUpUrl="#/sign-up"
            fallbackRedirectUrl="/"
          />
        </div>
      </section>
    </main>
  );
}

export function MissingClerkConfig() {
  const copy = translations.en;

  return (
    <main className="login-page theme-dark">
      <section className="missing-config-panel">
        <span className="material-symbols-outlined brand-icon">terminal</span>
        <p className="label-caps">Git Voor</p>
        <h1>{copy.auth.missingTitle}</h1>
        <p>{copy.auth.missingDescription}</p>
      </section>
    </main>
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

function SettingsPage({ copy, onDeleteAccount, onDeleteRepository, onSave, onUpdate, saveStatus, settings }) {
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
            <TextField disabled help={copy.settings.emailLocked} label={copy.settings.email} type="email" value={settings.email} onChange={() => {}} />
          </div>
        </SettingsPanel>

        <SettingsPanel eyebrow={copy.settings.dangerEyebrow} title={copy.settings.dangerTitle}>
          <div className="danger-actions">
            <p>{copy.settings.dangerDescription}</p>
            <div className="danger-action-row">
              <div>
                <strong>{copy.settings.removeRepository}</strong>
                <span>{copy.settings.removeRepositoryHelp}</span>
              </div>
              <button className="danger-button" type="button" onClick={onDeleteRepository}>{copy.settings.removeRepository}</button>
            </div>
            <div className="danger-action-row">
              <div>
                <strong>{copy.settings.deleteAccount}</strong>
                <span>{copy.settings.deleteAccountHelp}</span>
              </div>
              <button className="danger-button" type="button" onClick={onDeleteAccount}>{copy.settings.deleteAccount}</button>
            </div>
          </div>
        </SettingsPanel>
      </div>

      <div className="settings-actions">
        {saveStatus ? <span className="settings-status">{saveStatus}</span> : null}
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

function TextField({ disabled = false, help, label, onChange, type = "text", value }) {
  return (
    <label className="field-label">
      {label}
      <input disabled={disabled} type={type} value={value} onChange={(event) => onChange(event.target.value)} />
      {help ? <span className="field-help">{help}</span> : null}
    </label>
  );
}
