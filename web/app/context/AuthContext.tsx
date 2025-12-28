import { isAxiosError } from "axios";
import React, { createContext, useContext, useEffect, useState } from "react";
import ReactGA from "react-ga4";

import UserService from "../api/user";
import type { SelfUser } from "../types/models/user";
import type { UnitId } from "../types/models/base";
import type { Booking } from "../types/models/booking";

export interface AuthContextData {
  user: SelfUser;
  activeBookings: Record<UnitId, Booking>;
  pendingBookings: Record<UnitId, Booking[]>;
}

const AuthRefreshContext = createContext<() => Promise<void>>(async () => {});

const AuthContext = createContext<AuthContextData | null | undefined>(
  undefined,
);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [data, setData] = useState<AuthContextData | null | undefined>(
    undefined,
  );

  const fetchMe = async () => {
    try {
      const result = await UserService.me();
      ReactGA.set({
        user_id: result.data.user.id,
        traffic_type: result.data.user.isAdministrator ? "internal" : undefined,
      });
      setData({
        user: result.data.user,
        activeBookings: result.data.activeBookings,
        pendingBookings: result.data.pendingBookings,
      });
    } catch (error) {
      if (isAxiosError(error)) {
        if (error.status === 401) {
          setData(null);
        }
      }
    }
  };

  useEffect(() => {
    window.addEventListener("focus", fetchMe);
    fetchMe();

    return () => {
      window.removeEventListener("focus", fetchMe);
    };
  }, []);

  return (
    <AuthContext.Provider value={data}>
      <AuthRefreshContext.Provider value={fetchMe}>
        {children}
      </AuthRefreshContext.Provider>
    </AuthContext.Provider>
  );
}

export const useAuthRefresh = () => useContext(AuthRefreshContext);
export const useAuth = () => useContext(AuthContext);
