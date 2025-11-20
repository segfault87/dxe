import React, { createContext, useContext } from "react";

export interface Environment {
  urlBase: string;
  kakaoSdkKey: string;
}

const EnvContext = createContext<Environment>({
  urlBase: import.meta.env.VITE_URL_BASE,
  kakaoSdkKey: import.meta.env.VITE_KAKAO_SDK_KEY,
});

export function EnvProvider({ children }: { children: React.ReactNode }) {
  const value = {
    urlBase: import.meta.env.VITE_URL_BASE,
    kakaoSdkKey: import.meta.env.VITE_KAKAO_SDK_KEY,
  };

  return <EnvContext.Provider value={value}>{children}</EnvContext.Provider>;
}

export const useEnv = () => useContext(EnvContext);
