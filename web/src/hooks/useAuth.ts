import { useCallback, useEffect, useState } from "react";
import { auth, type User } from "../lib/api";
import { sendHeartbeat } from "../lib/device";

export interface RegisterResult {
  pending: boolean; // account created but must verify email before login
  emailSent: boolean;
  user: User;
}

export interface AuthState {
  user: User | null;
  loading: boolean;
  emailConfigured: boolean; // whether the server has email delivery set up
  refresh: () => Promise<void>;
  login: (email: string, pw: string) => Promise<void>;
  register: (email: string, pw: string) => Promise<RegisterResult>;
  logout: () => Promise<void>;
}

export function useAuth(): AuthState {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [emailConfigured, setEmailConfigured] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const res = await auth.me();
      setEmailConfigured(!!res.emailConfigured);
      const u = res.authenticated && res.user ? res.user : null;
      setUser(u);
      if (u) void sendHeartbeat(); // update last-seen + anonymous aggregate
    } catch {
      setUser(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const login = useCallback(async (email: string, pw: string) => {
    const res = await auth.login(email, pw);
    setUser(res.user);
    void sendHeartbeat({ login: true });
  }, []);

  const register = useCallback(async (email: string, pw: string): Promise<RegisterResult> => {
    const res = await auth.register(email, pw);
    if (!res.pending) {
      setUser(res.user);
      void sendHeartbeat({ login: true });
    }
    return { pending: !!res.pending, emailSent: !!res.emailSent, user: res.user };
  }, []);

  const logout = useCallback(async () => {
    await auth.logout();
    setUser(null);
  }, []);

  return { user, loading, emailConfigured, refresh, login, register, logout };
}
