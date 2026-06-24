import { useCallback, useEffect, useState } from "react";
import { auth, type User } from "../lib/api";

export interface AuthState {
  user: User | null;
  loading: boolean;
  refresh: () => Promise<void>;
  login: (id: string, pw: string) => Promise<void>;
  register: (id: string, pw: string, email?: string) => Promise<void>;
  logout: () => Promise<void>;
}

export function useAuth(): AuthState {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const res = await auth.me();
      setUser(res.authenticated && res.user ? res.user : null);
    } catch {
      setUser(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const login = useCallback(
    async (id: string, pw: string) => {
      const res = await auth.login(id, pw);
      setUser(res.user);
    },
    [],
  );

  const register = useCallback(async (id: string, pw: string, email?: string) => {
    const res = await auth.register(id, pw, email);
    setUser(res.user);
  }, []);

  const logout = useCallback(async () => {
    await auth.logout();
    setUser(null);
  }, []);

  return { user, loading, refresh, login, register, logout };
}
