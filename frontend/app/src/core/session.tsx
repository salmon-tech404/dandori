import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
  FC,
} from "react";
import { apiClient, getSafeErrorMessage } from "./api";

export interface User {
  id: string;
  username: string;
  system_role: string;
}

export interface SessionContextType {
  user: User | null;
  loading: boolean;
  error: string | null;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshSession: () => Promise<void>;
}

export const SessionContext = createContext<SessionContextType | null>(null);

function useSessionAuth() {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const refreshSession = useCallback(async () => {
    try {
      setUser(await apiClient<User>("/api/v1/me"));
      setError(null);
    } catch {
      setUser(null);
    } finally {
      setLoading(false);
    }
  }, []);

  const login = useCallback(async (u: string, p: string) => {
    setError(null);
    try {
      const res = await apiClient<{ user: User }>("/api/v1/auth/login", {
        method: "POST",
        body: JSON.stringify({ username: u, password: p }),
      });
      setUser(res.user);
    } catch (err) {
      setError(getSafeErrorMessage(err).userMessage);
      throw err;
    }
  }, []);

  const logout = useCallback(async () => {
    try {
      await apiClient("/api/v1/auth/logout", { method: "POST" });
    } catch {
      // Bỏ qua lỗi mạng
    } finally {
      setUser(null);
      setError(null);
    }
  }, []);

  return { user, setUser, loading, error, refreshSession, login, logout };
}

export const SessionProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const auth = useSessionAuth();
  const { refreshSession, setUser } = auth;

  useEffect(() => {
    refreshSession();
    const handleUnauthorized = () => setUser(null);
    window.addEventListener("dandori:unauthorized", handleUnauthorized);
    return () => {
      window.removeEventListener("dandori:unauthorized", handleUnauthorized);
    };
  }, [refreshSession, setUser]);

  return (
    <SessionContext.Provider value={auth}>
      {children}
    </SessionContext.Provider>
  );
};

export function useSession(): SessionContextType {
  const context = useContext(SessionContext);
  if (!context) {
    throw new Error("useSession phải được dùng bên trong SessionProvider");
  }
  return context;
}
