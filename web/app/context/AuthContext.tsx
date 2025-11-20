import { isAxiosError } from "axios";
import React, { createContext, useContext, useEffect, useState } from "react";

import UserService from "../api/user";
import type { SelfUser } from "../types/models/user";
import type { UnitId } from "../types/models/base";
import type { Booking } from "../types/models/booking";

export interface AuthContextData {
  user: SelfUser;
  activeBookings: Record<UnitId, Booking>;
  pendingBookings: Record<UnitId, Booking[]>;
}

const AuthContext = createContext<AuthContextData | null | undefined>(
  undefined,
);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [data, setData] = useState<AuthContextData | null | undefined>(
    undefined,
  );

  useEffect(() => {
    const fetchMe = async () => {
      try {
        const result = await UserService.me();
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

    window.addEventListener("focus", fetchMe);
    fetchMe();

    return () => {
      window.removeEventListener("focus", fetchMe);
    };
  }, []);

  return <AuthContext.Provider value={data}>{children}</AuthContext.Provider>;
}

export const useAuth = () => useContext(AuthContext);
