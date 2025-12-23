"use client";

import { useEffect } from "react";

import type { Environment } from "../context/EnvContext";
import { useEnv } from "../context/EnvContext";
import generateRandomString from "../lib/random";

declare global {
  interface AuthorizationSettings {
    redirectUri?: string;
    state?: string;
    scope?: string;
    prompt?: string;
    loginHint?: string;
    nonce?: string;
    throughTalk?: boolean;
  }

  interface Window {
    Kakao: {
      init: (key: string) => void;
      isInitialized: () => boolean;
      Auth: {
        authorize: (settings: AuthorizationSettings) => void;
        setAccessToken: (token: string, persist?: boolean) => void;
      };
    };
  }
}

export function isKakaoWebView() {
  return window.navigator.userAgent.indexOf("KAKAOTALK") >= 0;
}

export function kakaoInAppLogin(env: Environment, redirectTo: string | null) {
  window.Kakao.Auth.authorize({
    redirectUri: `${env.urlBase}/api/auth/kakao/redirect`,
    scope: "profile_nickname,talk_calendar",
    state: JSON.stringify({ redirectTo, transparent: true }),
    nonce: generateRandomString(16),
    prompt: "none",
  });
}

export function kakaoLogin(env: Environment, redirectTo: string | null) {
  window.Kakao.Auth.authorize({
    redirectUri: `${env.urlBase}/api/auth/kakao/redirect`,
    scope: "profile_nickname,talk_calendar",
    state: JSON.stringify({ redirectTo }),
    nonce: generateRandomString(16),
    throughTalk: true,
  });
}

export default function KakaoSDK() {
  const env = useEnv();
  useEffect(() => {
    try {
      window.Kakao.init(env.kakaoSdkKey);
    } catch {
      // Suppress error
    }
  });

  return (
    <script
      src="https://t1.kakaocdn.net/kakao_js_sdk/2.7.8/kakao.min.js"
      integrity="sha384-WUSirVbD0ASvo37f3qQZuDap8wy76aJjmGyXKOYgPL/NdAs8HhgmPlk9dz2XQsNv"
      crossOrigin="anonymous"
    />
  );
}
