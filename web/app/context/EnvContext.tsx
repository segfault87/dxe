import React, { createContext, useContext } from "react";

export interface Environment {
  urlBase: string;
  kakaoSdkKey: string;
  gaMeasurementId: string;
  enableTossPayments: boolean;
  tossPaymentClientKey: string;
}

const EnvContext = createContext<Environment>({
  urlBase: import.meta.env.VITE_URL_BASE,
  kakaoSdkKey: import.meta.env.VITE_KAKAO_SDK_KEY,
  gaMeasurementId: import.meta.env.VITE_GA_MEASUREMENT_ID,
  enableTossPayments: import.meta.env.VITE_ENABLE_TOSS_PAYMENTS === "true",
  tossPaymentClientKey: import.meta.env.VITE_TOSS_PAYMENT_CLIENT_KEY,
});

export function EnvProvider({ children }: { children: React.ReactNode }) {
  const value = {
    urlBase: import.meta.env.VITE_URL_BASE,
    kakaoSdkKey: import.meta.env.VITE_KAKAO_SDK_KEY,
    gaMeasurementId: import.meta.env.VITE_GA_MEASUREMENT_ID,
    enableTossPayments: import.meta.env.VITE_ENABLE_TOSS_PAYMENTS === "true",
    tossPaymentClientKey: import.meta.env.VITE_TOSS_PAYMENT_CLIENT_KEY,
  };

  return <EnvContext.Provider value={value}>{children}</EnvContext.Provider>;
}

export const useEnv = () => useContext(EnvContext);
