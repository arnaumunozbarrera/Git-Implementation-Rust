import { SignIn, SignUp, SignedIn, SignedOut, useAuth, useClerk, useUser } from "@clerk/clerk-react";
import { useEffect, useState } from "react";
import {
  deleteAccountRecords,
  deleteRepository,
  fetchActivityFeed,
  fetchAnalyticsOverview,
  fetchBranches,
  fetchCommitGraph,
  fetchCommitHistory,
  fetchRepositories,
  initRepository,
  updateAccountProfile,
} from "./api.js";
import { SystemHealthCard } from "./components/SystemHealthCard.jsx";

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
      cliLoginComplete: "Local sync complete. You can return to the terminal.",
      cliLoginError: "Unable to sync server locally.",
      cliLoginPending: "Syncing server locally...",
      missingTitle: "Clerk is not configured",
      missingDescription: "Set VITE_CLERK_PUBLISHABLE_KEY in the frontend environment to enable login.",
    },
    service: {
      eyebrow: "Service Status",
      title: "Service is down",
      description: "Repository data is unavailable because the backend service cannot be reached.",
      retry: "Retry",
      signedInAs: "Signed in as",
    },
    repository: {
      repository: "Repository",
      branch: "Branch",
      loading: "Loading...",
      noData: "No data available",
      noBranches: "No branches",
      private: "private",
      public: "public",
      create: "Create repository",
      createTitle: "Create remote repository",
      createDescription: "Configure the remote repository before cloning it.",
      name: "Name",
      defaultBranch: "Default branch",
      description: "Description",
      readmePath: "README path",
      cloneAction: "Cloning it",
      isPrivate: "Private repository",
      cancel: "Cancel",
      submit: "Create",
      creating: "Creating...",
      created: "Repository created",
      createError: "Unable to create repository",
    },
    nav: {
      overview: "Overview",
      activity: "Activity",
      branches: "Branches",
      sync: "Sync Monitor",
      settings: "Settings",
    },
    pages: {
      home: {
        eyebrow: "System Control",
        title: "Home",
        description: "Backend availability and service telemetry.",
        health: {
          eyebrow: "System Health",
          title: "Service Health",
          loading: "Checking...",
          noData: "No health data available",
          uptime: "Uptime",
          services: "Services",
          latestEvent: "Latest event",
          statuses: {
            degraded: "Degraded",
            down: "Down",
            healthy: "Healthy",
            unknown: "Unknown",
            warning: "Warning",
          },
        },
      },
      overview: {
        eyebrow: "Repository Control Plane",
        title: "Overview",
        description: "This workspace is reserved for repository summary modules.",
        stats: {
          totalCommits: "Total Commits",
          contributors: "contributors",
          lastActivity: "Last Activity",
          lastActivityContext: "latest repository event",
          repositorySize: "Repository Size",
          objects: "objects",
          loading: "Loading...",
          noData: "No data available",
        },
        recent: {
          title: "Recent Activity",
          eyebrow: "Latest 10 activities",
          contributor: "Contributor",
          loading: "Loading activity...",
          noData: "No recent activity available",
        },
        signals: {
          title: "Branch Commit Distribution",
          eyebrow: "Commit composition",
          commits: "commits",
          nextUpdate: "Next analysis/update: after the next remote sync.",
          noData: "No branch commit data available",
        },
      },
      activity: {
        eyebrow: "Repository Events",
        title: "Activity",
        description: "This workspace is reserved for commit and access activity streams.",
      },
      branches: {
        eyebrow: "Version Graph",
        title: "Branches",
        description: "Branch heads, recent commits, and the visible commit chain for the active repository.",
        head: "Head",
        created: "Created",
        commits: "Recent commits",
        graph: "Commit graph",
        noHead: "No head commit",
        noCommits: "No commits available",
        loading: "Loading branch data...",
        noData: "No branch data available",
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
      profileSaved: "Profile saved",
      profileError: "Unable to save profile",
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
      cliLoginComplete: "Sincronizacion local completada. Puedes volver al terminal.",
      cliLoginError: "No se pudo sincronizar el servidor localmente.",
      cliLoginPending: "Sincronizando servidor localmente...",
      missingTitle: "Clerk no esta configurado",
      missingDescription: "Define VITE_CLERK_PUBLISHABLE_KEY en el entorno del frontend para activar el inicio de sesion.",
    },
    service: {
      eyebrow: "Estado del servicio",
      title: "El servicio no esta disponible",
      description: "Los datos del repositorio no estan disponibles porque no se puede acceder al backend.",
      retry: "Reintentar",
      signedInAs: "Sesion iniciada como",
    },
    repository: {
      repository: "Repositorio",
      branch: "Rama",
      loading: "Cargando...",
      noData: "No hay datos disponibles",
      noBranches: "Sin ramas",
      private: "privado",
      public: "publico",
      create: "Crear repositorio",
      createTitle: "Crear repositorio remoto",
      createDescription: "Configura el repositorio remoto antes de clonarlo.",
      name: "Nombre",
      defaultBranch: "Rama principal",
      description: "Descripcion",
      readmePath: "Ruta README",
      cloneAction: "Clonandolo",
      isPrivate: "Repositorio privado",
      cancel: "Cancelar",
      submit: "Crear",
      creating: "Creando...",
      created: "Repositorio creado",
      createError: "No se pudo crear el repositorio",
    },
    nav: {
      overview: "Resumen",
      activity: "Actividad",
      branches: "Ramas",
      sync: "Monitor de sincronizacion",
      settings: "Ajustes",
    },
    pages: {
      home: {
        eyebrow: "Control del sistema",
        title: "Inicio",
        description: "Disponibilidad del backend y telemetria de servicios.",
        health: {
          eyebrow: "Salud del sistema",
          title: "Estado del servicio",
          loading: "Comprobando...",
          noData: "No hay datos de salud disponibles",
          uptime: "Tiempo activo",
          services: "Servicios",
          latestEvent: "Ultimo evento",
          statuses: {
            degraded: "Degradado",
            down: "Caido",
            healthy: "Correcto",
            unknown: "Desconocido",
            warning: "Aviso",
          },
        },
      },
      overview: {
        eyebrow: "Panel de control del repositorio",
        title: "Resumen",
        description: "Este espacio esta reservado para los modulos de resumen del repositorio.",
        stats: {
          totalCommits: "Commits totales",
          contributors: "colaboradores",
          lastActivity: "Ultima actividad",
          lastActivityContext: "ultimo evento del repositorio",
          repositorySize: "Tamano del repositorio",
          objects: "objetos",
          loading: "Cargando...",
          noData: "No hay datos disponibles",
        },
        recent: {
          title: "Actividad reciente",
          eyebrow: "Ultimas 10 actividades",
          contributor: "Colaborador",
          loading: "Cargando actividad...",
          noData: "No hay actividad reciente disponible",
        },
        signals: {
          title: "Distribucion de commits por rama",
          eyebrow: "Composicion de commits",
          commits: "commits",
          nextUpdate: "Proximo analisis/actualizacion: despues de la siguiente sincronizacion remota.",
          noData: "No hay datos de commits por rama disponibles",
        },
      },
      activity: {
        eyebrow: "Eventos del repositorio",
        title: "Actividad",
        description: "Este espacio esta reservado para commits y flujos de actividad de acceso.",
      },
      branches: {
        eyebrow: "Grafo de versiones",
        title: "Ramas",
        description: "Cabeceras de rama, commits recientes y cadena visible para el repositorio activo.",
        head: "Cabecera",
        created: "Creada",
        commits: "Commits recientes",
        graph: "Grafo de commits",
        noHead: "Sin commit de cabecera",
        noCommits: "No hay commits disponibles",
        loading: "Cargando datos de rama...",
        noData: "No hay datos de rama disponibles",
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
      profileSaved: "Perfil guardado",
      profileError: "No se pudo guardar el perfil",
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
  language: "en",
  theme: "dark",
  displayName: "",
  username: "",
  email: "",
};

let cliLoginAttemptStarted = false;

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

function displayNameFromUser(user, settings) {
  return settings.displayName || user?.fullName || user?.username || user?.primaryEmailAddress?.emailAddress || "";
}

function emailFromUser(user, settings) {
  return settings.email || user?.primaryEmailAddress?.emailAddress || "";
}

function visibilityFromRepository(repo) {
  if (!repo) {
    return "";
  }

  return repo.is_private ? "private" : "public";
}

function repositoryIdFromName(name) {
  return String(name || "")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function NativeSelect({ children, className = "", ...props }) {
  return (
    <select className={`native-select ${className}`.trim()} {...props}>
      {children}
    </select>
  );
}

function NativeSelectOption({ children, ...props }) {
  return <option {...props}>{children}</option>;
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
        <CliLoginBridge copy={copy.auth} />
        <AuthenticatedShell copy={copy} settings={settings} setSettings={setSettings} />
      </SignedIn>
    </>
  );
}

function cliLoginPortFromUrl() {
  const params = new URLSearchParams(window.location.search);
  const port = Number(params.get("cli_login_port"));
  if (!Number.isInteger(port) || port < 1024 || port > 65535) {
    return null;
  }
  return port;
}

function CliLoginBridge({ copy }) {
  const { getToken } = useAuth();
  const [status, setStatus] = useState(() => (cliLoginPortFromUrl() ? "pending" : "idle"));

  useEffect(() => {
    const port = cliLoginPortFromUrl();
    if (!port) {
      return;
    }
    if (cliLoginAttemptStarted) {
      return;
    }

    let active = true;
    cliLoginAttemptStarted = true;

    async function completeCliLogin() {
      try {
        const token = await getToken();
        if (!token) {
          throw new Error("Clerk did not return a token");
        }

        const response = await fetch(`http://127.0.0.1:${port}/auth-token`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ token }),
        });

        if (!response.ok) {
          throw new Error(`CLI callback failed with ${response.status}`);
        }

        if (active) {
          setStatus("complete");
          const url = new URL(window.location.href);
          url.searchParams.delete("cli_login_port");
          window.history.replaceState({}, "", `${url.pathname}${url.search}${url.hash}`);
        }
      } catch {
        if (active) {
          setStatus("error");
        }
      }
    }

    completeCliLogin();

    return () => {
      active = false;
    };
  }, [getToken]);

  useEffect(() => {
    if (status === "idle") {
      return;
    }

    const timeout = window.setTimeout(() => {
      setStatus("idle");
    }, status === "pending" ? 5000 : 3500);

    return () => window.clearTimeout(timeout);
  }, [status]);

  if (status === "idle") {
    return null;
  }

  const message = status === "complete"
    ? copy.cliLoginComplete
    : status === "error"
      ? copy.cliLoginError
      : copy.cliLoginPending;

  return (
    <div className={`cli-login-toast cli-login-toast-${status}`} role="status">
      <span className="material-symbols-outlined" aria-hidden="true">
        {status === "error" ? "error" : status === "complete" ? "check_circle" : "sync"}
      </span>
      <span>{message}</span>
    </div>
  );
}

function AuthenticatedShell({ copy, settings, setSettings }) {
  const { openSignIn, signOut } = useClerk();
  const { getToken, isLoaded, isSignedIn } = useAuth();
  const { user } = useUser();
  const [activePage, setActivePage] = useState("overview");
  const [accountMenuOpen, setAccountMenuOpen] = useState(false);
  const [saveStatus, setSaveStatus] = useState("");
  const [repositoryState, setRepositoryState] = useState({
    status: "loading",
    repositories: [],
    error: null,
  });
  const [selectedRepositoryId, setSelectedRepositoryId] = useState("");
  const [createRepoOpen, setCreateRepoOpen] = useState(false);
  const [createRepoStatus, setCreateRepoStatus] = useState("");
  const [createRepoForm, setCreateRepoForm] = useState({
    name: "",
    defaultBranch: "main",
    description: "",
    readmePath: "README.md",
    isPrivate: true,
    cloneAfterCreate: true,
  });
  const [branchState, setBranchState] = useState({
    status: "idle",
    branches: [],
    error: null,
  });
  const [selectedBranchName, setSelectedBranchName] = useState("");

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

  const loadRepositories = () => {
    if (!isLoaded || !isSignedIn) {
      return;
    }

    setRepositoryState({ status: "loading", repositories: [], error: null });
    fetchRepositories(getToken)
      .then((repositories) => {
        const normalizedRepositories = Array.isArray(repositories) ? repositories : [];
        setRepositoryState({
          status: "ready",
          repositories: normalizedRepositories,
          error: null,
        });
        setSelectedRepositoryId((current) => {
          if (current && normalizedRepositories.some((repository) => repository.id === current)) {
            return current;
          }

          return normalizedRepositories[0]?.id ?? "";
        });
      })
      .catch((error) => {
        setRepositoryState({ status: "unavailable", repositories: [], error: error.message });
        setSelectedRepositoryId("");
      });
  };

  useEffect(() => {
    if (isLoaded && isSignedIn) {
      loadRepositories();
    }
  }, [getToken, isLoaded, isSignedIn]);

  useEffect(() => {
    let active = true;

    if (!selectedRepositoryId || !isLoaded || !isSignedIn) {
      setBranchState({ status: "idle", branches: [], error: null });
      setSelectedBranchName("");
      return () => {
        active = false;
      };
    }

    setBranchState({ status: "loading", branches: [], error: null });
    fetchBranches(selectedRepositoryId, getToken)
      .then((branches) => {
        if (!active) {
          return;
        }

        const normalizedBranches = Array.isArray(branches) ? branches : [];
        setBranchState({ status: "ready", branches: normalizedBranches, error: null });
        setSelectedBranchName((current) => {
          if (current && normalizedBranches.some((branch) => branch.name === current)) {
            return current;
          }

          const selectedRepository = repositoryState.repositories.find((repository) => repository.id === selectedRepositoryId);
          return (
            normalizedBranches.find((branch) => branch.name === selectedRepository?.default_branch)?.name ??
            normalizedBranches[0]?.name ??
            ""
          );
        });
      })
      .catch((error) => {
        if (active) {
          setBranchState({ status: "unavailable", branches: [], error: error.message });
          setSelectedBranchName("");
        }
      });

    return () => {
      active = false;
    };
  }, [getToken, isLoaded, isSignedIn, repositoryState.repositories, selectedRepositoryId]);

  const updateSetting = (key, value) => {
    setSaveStatus("");
    setSettings((current) => ({ ...current, [key]: value }));
  };

  const saveSettings = async () => {
    const { initials, ...settingsToSave } = settings;
    localStorage.setItem("gitVoorSettings", JSON.stringify(settingsToSave));

    try {
      await updateAccountProfile({
        username: settings.username || null,
        email: settings.email || user?.primaryEmailAddress?.emailAddress || null,
      }, getToken);

      if (user) {
        try {
          const [firstName, ...lastNameParts] = settings.displayName.trim().split(/\s+/);
          await user.update({
            firstName: firstName || undefined,
            lastName: lastNameParts.join(" ") || undefined,
            username: settings.username || undefined,
          });
        } catch {
          // The backend profile is the source for dashboard activity names.
        }
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
    const activeRepository = repositoryState.repositories.find((repository) => repository.id === selectedRepositoryId) ?? repositoryState.repositories[0];
    if (!activeRepository) {
      setSaveStatus(copy.service.description);
      return;
    }

    if (!window.confirm(copy.settings.confirmRemoveRepository)) {
      return;
    }

    try {
      await deleteRepository(activeRepository.id, getToken);
      loadRepositories();
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

  const updateCreateRepoForm = (key, value) => {
    setCreateRepoStatus("");
    setCreateRepoForm((current) => ({ ...current, [key]: value }));
  };

  const handleCreateRepository = async (event) => {
    event.preventDefault();
    setCreateRepoStatus("loading");

    try {
      const name = createRepoForm.name.trim();
      const repoId = repositoryIdFromName(name);
      if (!repoId) {
        throw new Error("Repository name must include letters or numbers");
      }
      await initRepository(
        {
          repo_id: repoId,
          name,
          owner_id: user?.id ?? "",
          default_branch: createRepoForm.defaultBranch.trim() || "main",
          is_private: createRepoForm.isPrivate,
          description: createRepoForm.description.trim() || null,
          readme_path: createRepoForm.readmePath.trim() || null,
          tags: null,
          theme: null,
          head: null,
          objects: null,
        },
        getToken,
      );

      setCreateRepoStatus("created");
      setCreateRepoOpen(false);
      setCreateRepoForm({
        name: "",
        defaultBranch: "main",
        description: "",
        readmePath: "README.md",
        isPrivate: true,
        cloneAfterCreate: true,
      });
      loadRepositories();
      setSelectedRepositoryId(repoId);
      setSaveStatus(copy.repository.created);
    } catch {
      setCreateRepoStatus("error");
    }
  };

  const appClassName = `app-shell theme-${settings.theme}`;
  const accountName = displayNameFromUser(user, settings);
  const accountEmail = emailFromUser(user, settings);
  const profileInitials = initialsFromUsername(settings.username || accountName || accountEmail);
  const activeRepository = repositoryState.repositories.find((repository) => repository.id === selectedRepositoryId) ?? repositoryState.repositories[0] ?? null;
  const activeBranch = branchState.branches.find((branch) => branch.name === selectedBranchName) ?? null;
  const repositoryCopy = copy.repository;
  const repoVisibility = activeRepository ? repositoryCopy[visibilityFromRepository(activeRepository)] : repositoryCopy.noData;
  const backendUnavailable = repositoryState.status === "unavailable";

  return (
    <div className={appClassName}>
      <aside className="side-nav" aria-label="Primary">
        <button
          className={`brand-block ${activePage === "home" ? "active" : ""}`}
          onClick={() => setActivePage("home")}
          type="button"
        >
          <span className="material-symbols-outlined brand-icon">terminal</span>
          <div>
            <div className="brand-title">Git Voor</div>
          </div>
        </button>

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
          <span className={`visibility-pill visibility-${visibilityFromRepository(activeRepository) || "unknown"}`}>
            {repoVisibility}
          </span>
          <label className="context-select-label">
            <span className="context-select-title">{repositoryCopy.repository}</span>
            <span className="context-select-control">
              <NativeSelect
                aria-label={repositoryCopy.repository}
                disabled={repositoryState.status !== "ready" || repositoryState.repositories.length === 0}
                value={activeRepository?.id ?? ""}
                onChange={(event) => setSelectedRepositoryId(event.target.value)}
              >
                {repositoryState.repositories.length === 0 ? (
                  <NativeSelectOption value="">{repositoryState.status === "loading" ? repositoryCopy.loading : repositoryCopy.noData}</NativeSelectOption>
                ) : (
                  repositoryState.repositories.map((repository) => (
                    <NativeSelectOption key={repository.id} value={repository.id}>
                      {repository.name || repository.id}
                    </NativeSelectOption>
                  ))
                )}
              </NativeSelect>
            </span>
          </label>
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
                  <strong>{accountName || copy.service.signedInAs}</strong>
                  <span>{accountEmail || user?.id}</span>
                </div>
                <button type="button" onClick={handleChangeAccount}>{copy.account.changeAccount}</button>
                <button type="button" onClick={handleLogout}>{copy.account.logout}</button>
              </div>
            ) : null}
          </div>
        </div>
      </header>

      <main className="main-canvas">
        {backendUnavailable ? (
          <ServiceDownPage
            accountEmail={accountEmail}
            accountName={accountName}
            copy={copy.service}
            onRetry={loadRepositories}
          />
        ) : activePage === "home" ? (
          <HomePage page={copy.pages.home} />
        ) : activePage === "settings" ? (
          <SettingsPage
            copy={copy}
            onDeleteAccount={handleDeleteAccount}
            onDeleteRepository={handleDeleteRepository}
            onSave={saveSettings}
            onUpdate={updateSetting}
            saveStatus={saveStatus}
            settings={settings}
          />
        ) : activePage === "overview" ? (
          <OverviewPage getToken={getToken} page={copy.pages.overview} repoId={activeRepository?.id} />
        ) : activePage === "branches" ? (
          <BranchesPage
            branch={activeBranch}
            branches={branchState.branches}
            branchName={selectedBranchName}
            getToken={getToken}
            onSelectBranch={setSelectedBranchName}
            page={copy.pages.branches}
            repoId={activeRepository?.id}
            status={branchState.status}
          />
        ) : (
          <EmptySection page={copy.pages[activePage]} />
        )}
      </main>

      <button
        className="create-repo-fab"
        type="button"
        aria-label={repositoryCopy.create}
        title={repositoryCopy.create}
        onClick={() => {
          setCreateRepoStatus("");
          setCreateRepoOpen(true);
        }}
      >
        <span className="material-symbols-outlined" aria-hidden="true">add</span>
      </button>

      {createRepoOpen ? (
        <CreateRepositoryModal
          copy={repositoryCopy}
          form={createRepoForm}
          onClose={() => setCreateRepoOpen(false)}
          onSubmit={handleCreateRepository}
          onUpdate={updateCreateRepoForm}
          status={createRepoStatus}
        />
      ) : null}
    </div>
  );
}

function LoginPage({ copy, theme }) {
  const isSignUp = window.location.hash.startsWith("#/sign-up");
  const redirectUrl = `${window.location.pathname}${window.location.search}`;
  const clerkProps = {
    appearance: clerkAppearance,
    fallbackRedirectUrl: redirectUrl,
    routing: "hash",
    signInUrl: redirectUrl,
    signUpUrl: `${redirectUrl}#/sign-up`,
  };

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
          {isSignUp ? <SignUp {...clerkProps} /> : <SignIn {...clerkProps} />}
        </div>
      </section>
    </main>
  );
}

function HomePage({ page }) {
  return (
    <section className="workspace-section">
      <div className="landing-heading">
        <p className="label-caps">{page.eyebrow}</p>
        <h1>{page.title}</h1>
        <p>{page.description}</p>
      </div>

      <SystemHealthCard copy={page.health} />
    </section>
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

function ServiceDownPage({ accountEmail, accountName, copy, onRetry }) {
  return (
    <section className="service-down-page" aria-live="polite">
      <span className="material-symbols-outlined service-down-icon" aria-hidden="true">cloud_off</span>
      <p className="label-caps">{copy.eyebrow}</p>
      <h1>{copy.title}</h1>
      <p>{copy.description}</p>
      {accountName || accountEmail ? (
        <div className="service-account">
          <span>{copy.signedInAs}</span>
          <strong>{accountName || accountEmail}</strong>
          {accountName && accountEmail ? <small>{accountEmail}</small> : null}
        </div>
      ) : null}
      <button className="secondary-button" type="button" onClick={onRetry}>{copy.retry}</button>
    </section>
  );
}

function CreateRepositoryModal({ copy, form, onClose, onSubmit, onUpdate, status }) {
  const isCreating = status === "loading";

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="modal-panel create-repo-modal" role="dialog" aria-modal="true" aria-labelledby="create-repo-title">
        <header className="modal-header">
          <div>
            <p className="label-caps">{copy.cloneAction}</p>
            <h2 id="create-repo-title">{copy.createTitle}</h2>
          </div>
          <button className="icon-button" type="button" aria-label={copy.cancel} onClick={onClose}>
            <span className="material-symbols-outlined" aria-hidden="true">close</span>
          </button>
        </header>

        <form className="modal-form" onSubmit={onSubmit}>
          <p className="modal-description">{copy.createDescription}</p>
          <div className="form-grid">
            <label className="field-label">
              {copy.name}
              <input required value={form.name} onChange={(event) => onUpdate("name", event.target.value)} />
            </label>
            <label className="field-label">
              {copy.defaultBranch}
              <input required value={form.defaultBranch} onChange={(event) => onUpdate("defaultBranch", event.target.value)} />
            </label>
            <label className="field-label">
              {copy.readmePath}
              <input value={form.readmePath} onChange={(event) => onUpdate("readmePath", event.target.value)} />
            </label>
          </div>

          <label className="field-label">
            {copy.description}
            <input value={form.description} onChange={(event) => onUpdate("description", event.target.value)} />
          </label>

          <div className="modal-toggle-row">
            <label>
              <input checked={form.isPrivate} type="checkbox" onChange={(event) => onUpdate("isPrivate", event.target.checked)} />
              <span>{copy.isPrivate}</span>
            </label>
            <label>
              <input checked={form.cloneAfterCreate} type="checkbox" onChange={(event) => onUpdate("cloneAfterCreate", event.target.checked)} />
              <span>{copy.cloneAction}</span>
            </label>
          </div>

          {status === "error" ? <p className="modal-status modal-status-error">{copy.createError}</p> : null}

          <footer className="modal-actions">
            <button className="secondary-button" type="button" onClick={onClose}>{copy.cancel}</button>
            <button className="primary-button" type="submit" disabled={isCreating}>
              {isCreating ? copy.creating : copy.submit}
            </button>
          </footer>
        </form>
      </section>
    </div>
  );
}

function compactNumber(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) {
    return "";
  }

  return new Intl.NumberFormat("en", { notation: "compact" }).format(number);
}

function formatBytes(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) {
    return "";
  }

  if (number === 0) {
    return "0 B";
  }

  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(Math.floor(Math.log(number) / Math.log(1024)), units.length - 1);
  const amount = number / 1024 ** index;
  return `${amount >= 10 || index === 0 ? amount.toFixed(0) : amount.toFixed(1)} ${units[index]}`;
}

function formatRelativeTime(value) {
  if (!value) {
    return "";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "";
  }

  const seconds = Math.round((date.getTime() - Date.now()) / 1000);
  const divisions = [
    { amount: 60, unit: "second" },
    { amount: 60, unit: "minute" },
    { amount: 24, unit: "hour" },
    { amount: 7, unit: "day" },
    { amount: 4.345, unit: "week" },
    { amount: 12, unit: "month" },
    { amount: Number.POSITIVE_INFINITY, unit: "year" },
  ];

  let duration = seconds;
  for (const division of divisions) {
    if (Math.abs(duration) < division.amount) {
      return new Intl.RelativeTimeFormat("en", { numeric: "auto" }).format(Math.round(duration), division.unit);
    }
    duration /= division.amount;
  }

  return "";
}

function getLatestActivity(data) {
  const dates = [data?.last_push_at, data?.last_pull_at]
    .filter(Boolean)
    .map((value) => new Date(value))
    .filter((date) => !Number.isNaN(date.getTime()))
    .sort((left, right) => right.getTime() - left.getTime());

  return dates[0]?.toISOString() ?? null;
}

function activityAccentFor(value) {
  const palette = ["#58a6ff", "#7ee787", "#ffba42", "#d2a8ff", "#ff7b72", "#39c5cf", "#ffa657"];
  const seed = String(value || "unknown");
  let hash = 0;

  for (let index = 0; index < seed.length; index += 1) {
    hash = (hash + seed.charCodeAt(index) * (index + 1)) % palette.length;
  }

  return palette[hash];
}

function displayNameFromActor(actor) {
  return actor?.username || actor?.email || "";
}

function languageAccentFor(index) {
  const palette = ["#58a6ff", "#7ee787", "#ffba42", "#d2a8ff", "#ff7b72", "#39c5cf", "#ffa657", "#a5d6ff"];
  return palette[index % palette.length];
}

function formatDateTime(value) {
  if (!value) {
    return "";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

function hasOverviewData(data) {
  return Boolean(
    data &&
      Number.isFinite(Number(data.commits_count)) &&
      Number.isFinite(Number(data.contributors_count)) &&
      Number.isFinite(Number(data.repository_size_bytes)) &&
      Number.isFinite(Number(data.object_count)),
  );
}

function OverviewPage({ getToken, page, repoId }) {
  const stats = page.stats;
  const [state, setState] = useState({
    status: "loading",
    data: null,
  });
  const [activityState, setActivityState] = useState({
    status: "loading",
    items: [],
  });

  useEffect(() => {
    let active = true;

    if (!repoId) {
      setState({ status: "unavailable", data: null });
      setActivityState({ status: "unavailable", items: [] });
      return () => {
        active = false;
      };
    }

    setState({ status: "loading", data: null });
    setActivityState({ status: "loading", items: [] });
    fetchAnalyticsOverview(repoId, getToken)
      .then((data) => {
        if (active) {
          setState({ status: "ready", data });
        }
      })
      .catch(() => {
        if (active) {
          setState({ status: "unavailable", data: null });
        }
      });

    fetchActivityFeed(repoId, getToken, 10, "commit")
      .then((data) => {
        if (active) {
          setActivityState({
            status: "ready",
            items: Array.isArray(data?.items) ? data.items.slice(0, 10) : [],
          });
        }
      })
      .catch(() => {
        if (active) {
          setActivityState({ status: "unavailable", items: [] });
        }
      });

    return () => {
      active = false;
    };
  }, [getToken, repoId]);

  const data = state.data;
  const isReady = state.status === "ready" && hasOverviewData(data);
  const unavailableText = state.status === "loading" ? stats.loading : stats.noData;
  const latestActivity = isReady ? formatRelativeTime(getLatestActivity(data)) : "";

  return (
    <section className="workspace-section">
      <div className="landing-heading">
        <p className="label-caps">{page.eyebrow}</p>
        <h1>{page.title}</h1>
        <p>{page.description}</p>
      </div>

      <div className="overview-stat-grid" aria-label="Repository summary">
        <OverviewStatCard
          icon="insights"
          label={stats.totalCommits}
          value={isReady ? compactNumber(data.commits_count) : unavailableText}
          meta={isReady ? `${compactNumber(data.contributors_count)} ${stats.contributors}` : ""}
          tone={isReady ? "positive" : undefined}
        />
        <OverviewStatCard
          icon="schedule"
          label={stats.lastActivity}
          value={isReady && latestActivity ? latestActivity : unavailableText}
          meta={isReady && latestActivity ? stats.lastActivityContext : ""}
        />
        <OverviewStatCard
          icon="database"
          label={stats.repositorySize}
          value={isReady ? formatBytes(data.repository_size_bytes) : unavailableText}
          meta={isReady ? `${compactNumber(data.object_count)} ${stats.objects}` : ""}
        />
      </div>

      <div className="overview-detail-grid">
        <RecentActivityPanel copy={page.recent} items={activityState.items} status={activityState.status} />
        <RepositorySignalsPanel copy={page.signals} data={data} isReady={isReady} />
      </div>
    </section>
  );
}

function OverviewStatCard({ icon, label, meta, tone, value }) {
  return (
    <article className="overview-stat-card">
      <header className="overview-stat-header">
        <span>{label}</span>
        <span className="material-symbols-outlined" aria-hidden="true">{icon}</span>
      </header>
      <strong>{value}</strong>
      <p className={tone === "positive" ? "stat-meta stat-meta-positive" : "stat-meta"}>{meta}</p>
    </article>
  );
}

function RecentActivityPanel({ copy, items, status }) {
  const isLoading = status === "loading";
  const commitItems = items.filter((item) => item.action === "commit");

  return (
    <section className="overview-panel recent-activity-panel">
      <header className="overview-panel-header">
        <div>
          <h2>{copy.title}</h2>
        </div>
        <span className="material-symbols-outlined" aria-hidden="true">history</span>
      </header>

      {commitItems.length > 0 ? (
        <div className="recent-activity-list">
          {commitItems.map((item, index) => {
            const contributor = displayNameFromActor(item.actor);
            const accent = activityAccentFor(contributor);

            return (
              <article className="recent-activity-row" key={`${item.created_at}-${index}`} style={{ "--activity-accent": accent }}>
                <div className="activity-user-rail">
                  {contributor ? <span>{contributor}</span> : null}
                </div>
                <div className="activity-main">
                  <div className="activity-main-header">
                    <strong>{item.message}</strong>
                    <time dateTime={item.created_at}>{formatRelativeTime(item.created_at)}</time>
                  </div>
                  {contributor ? (
                    <div className="activity-meta-line">
                      <span>{copy.contributor}: {contributor}</span>
                    </div>
                  ) : null}
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <p className="overview-panel-empty">{isLoading ? copy.loading : copy.noData}</p>
      )}
    </section>
  );
}

function RepositorySignalsPanel({ copy, data, isReady }) {
  const branches = isReady && Array.isArray(data?.branch_commit_distribution)
    ? data.branch_commit_distribution
    : [];

  return (
    <section className="overview-panel repository-signals-panel">
      <header className="overview-panel-header">
        <div>
          <h2>{copy.title}</h2>
        </div>
        <span className="material-symbols-outlined" aria-hidden="true">data_usage</span>
      </header>

      <p className="distribution-update-message">{copy.nextUpdate}</p>

      {branches.length > 0 ? (
        <div className="language-distribution-list">
          {branches.map((item, index) => (
            <div
              className="language-distribution-row"
              key={item.branch}
              style={{ "--language-accent": languageAccentFor(index) }}
            >
              <strong className="language-percentage">{Number(item.percentage).toFixed(1)}%</strong>
              <div className="language-distribution-body">
                <div className="language-distribution-heading">
                  <span>{item.branch}</span>
                  <small>{compactNumber(item.total_count)} {copy.commits}</small>
                </div>
                <div className="language-progress-track" aria-hidden="true">
                  <span style={{ width: `${Math.max(0, Math.min(100, Number(item.percentage) || 0))}%` }} />
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <p className="overview-panel-empty">{copy.noData}</p>
      )}
    </section>
  );
}

function BranchesPage({ branch, branches, branchName, getToken, onSelectBranch, page, repoId, status }) {
  const [state, setState] = useState({
    status: "idle",
    graph: null,
    history: [],
  });

  useEffect(() => {
    let active = true;

    if (!repoId || !branchName) {
      setState({ status: "empty", graph: null, history: [] });
      return () => {
        active = false;
      };
    }

    setState({ status: "loading", graph: null, history: [] });
    Promise.all([
      fetchCommitGraph(repoId, branchName, getToken, 24),
      fetchCommitHistory(repoId, branchName, getToken, 6),
    ])
      .then(([graph, history]) => {
        if (active) {
          setState({
            status: "ready",
            graph,
            history: Array.isArray(history?.items) ? history.items : [],
          });
        }
      })
      .catch(() => {
        if (active) {
          setState({ status: "unavailable", graph: null, history: [] });
        }
      });

    return () => {
      active = false;
    };
  }, [branchName, getToken, repoId]);

  const isLoading = status === "loading" || state.status === "loading";
  const graphNodes = Array.isArray(state.graph?.nodes) ? state.graph.nodes : [];
  const hasData = branch && state.status === "ready";

  return (
    <section className="workspace-section">
      <div className="landing-heading">
        <p className="label-caps">{page.eyebrow}</p>
        <h1>{page.title}</h1>
        <p>{page.description}</p>
      </div>

      <div className="branch-layout">
        <section className="branch-summary-panel">
          <header className="branch-panel-header">
            <div>
              <p className="label-caps">{page.title}</p>
              <h2>{branchName || page.noData}</h2>
            </div>
            <span className="visibility-pill">{compactNumber(branches.length)}</span>
          </header>
          {branches.length > 0 ? (
            <div className="branch-chip-list" aria-label={page.title}>
              {branches.map((item) => (
                <button
                  className={`branch-chip ${item.name === branchName ? "active" : ""}`}
                  key={item.id}
                  onClick={() => onSelectBranch(item.name)}
                  type="button"
                >
                  <span className="material-symbols-outlined" aria-hidden="true">call_split</span>
                  <span>{item.name}</span>
                </button>
              ))}
            </div>
          ) : null}
          <div className="branch-facts">
            <div>
              <span>{page.head}</span>
              <strong>{branch?.last_commit_hash?.slice(0, 12) || (isLoading ? page.loading : page.noHead)}</strong>
            </div>
            <div>
              <span>{page.created}</span>
              <strong>{branch?.created_at ? formatDateTime(branch.created_at) : isLoading ? page.loading : page.noData}</strong>
            </div>
          </div>
        </section>

        <section className="branch-panel">
          <header className="branch-panel-header">
            <h2>{page.commits}</h2>
          </header>
          {hasData && state.history.length > 0 ? (
            <div className="commit-list">
              {state.history.map((commit) => (
                <article className="commit-row" key={commit.hash}>
                  <div>
                    <strong>{commit.message}</strong>
                    <span>{formatDateTime(commit.created_at)}</span>
                  </div>
                  <code>{commit.hash.slice(0, 10)}</code>
                </article>
              ))}
            </div>
          ) : (
            <p className="branch-empty">{isLoading ? page.loading : page.noCommits}</p>
          )}
        </section>

        <section className="branch-panel branch-graph-panel">
          <header className="branch-panel-header">
            <h2>{page.graph}</h2>
          </header>
          {hasData && graphNodes.length > 0 ? (
            <div className="graph-list">
              {graphNodes.map((node) => (
                <article className="graph-node" key={node.hash}>
                  <span className="graph-dot" aria-hidden="true" />
                  <div>
                    <strong>{node.message}</strong>
                    <span>
                      {node.hash.slice(0, 10)}
                      {node.branches?.length ? ` · ${node.branches.join(", ")}` : ""}
                    </span>
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <p className="branch-empty">{isLoading ? page.loading : page.noData}</p>
          )}
        </section>
      </div>
    </section>
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
              <NativeSelect value={settings.language} onChange={(event) => onUpdate("language", event.target.value)}>
                <NativeSelectOption value="en">{copy.settings.english}</NativeSelectOption>
                <NativeSelectOption value="es">{copy.settings.spanish}</NativeSelectOption>
              </NativeSelect>
            </label>
            <label className="field-label">
              {copy.settings.appearance}
              <NativeSelect value={settings.theme} onChange={(event) => onUpdate("theme", event.target.value)}>
                <NativeSelectOption value="dark">{copy.appearance.dark}</NativeSelectOption>
                <NativeSelectOption value="light">{copy.appearance.light}</NativeSelectOption>
              </NativeSelect>
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
