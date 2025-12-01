import React, { createContext, useContext } from "react";

export interface Environment {
  urlBase: string;
  kakaoSdkKey: string;
  gtmId: string;
  gaMeasurementId: string;
}

const EnvContext = createContext<Environment>({
  urlBase: import.meta.env.VITE_URL_BASE,
  kakaoSdkKey: import.meta.env.VITE_KAKAO_SDK_KEY,
  gtmId: import.meta.env.VITE_GTM_ID,
  gaMeasurementId: import.meta.env.VITE_GA_MEASUREMENT_ID,
});

export function EnvProvider({ children }: { children: React.ReactNode }) {
  const value = {
    urlBase: import.meta.env.VITE_URL_BASE,
    kakaoSdkKey: import.meta.env.VITE_KAKAO_SDK_KEY,
    gtmId: import.meta.env.VITE_GTM_ID,
    gaMeasurementId: import.meta.env.VITE_GA_MEASUREMENT_ID,
  };

  return <EnvContext.Provider value={value}>{children}</EnvContext.Provider>;
}

export const useEnv = () => useContext(EnvContext);
