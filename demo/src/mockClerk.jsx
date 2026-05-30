import { useCallback } from "react";

const demoUser = {
  id: "demo_user_01",
  fullName: "Maya Chen",
  username: "maya.chen",
  primaryEmailAddress: {
    emailAddress: "maya.chen@example.invalid",
  },
  delete: async () => ({ ok: true }),
  update: async () => ({ ok: true }),
};

export function SignedIn({ children }) {
  return children;
}

export function SignedOut() {
  return null;
}

export function useAuth() {
  const getToken = useCallback(async () => "demo-local-token", []);
  return {
    getToken,
    isLoaded: true,
    isSignedIn: true,
    userId: demoUser.id,
  };
}

export function useUser() {
  return {
    isLoaded: true,
    isSignedIn: true,
    user: demoUser,
  };
}

export function useClerk() {
  return {
    openSignIn: () => {},
    signOut: async () => {},
  };
}

function MockAuthCard({ mode }) {
  return (
    <div className="mock-auth-card clerk-card-box" aria-label={mode}>
      <div className="clerk-card">
        <h2 className="clerk-title">{mode === "signup" ? "Create demo account" : "Demo account"}</h2>
        <p className="clerk-subtitle">Authentication is simulated locally for this isolated demo.</p>
        <button className="clerk-primary-button" type="button">Continue</button>
      </div>
    </div>
  );
}

export function SignIn() {
  return <MockAuthCard mode="signin" />;
}

export function SignUp() {
  return <MockAuthCard mode="signup" />;
}
