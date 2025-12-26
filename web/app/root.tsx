import React, { useEffect } from "react";
import ReactGA from "react-ga4";
import Modal from "react-modal";
import {
  Outlet,
  Links,
  Meta,
  Scripts,
  ScrollRestoration,
  useNavigate,
} from "react-router";

import type { Route } from "./+types/root";
import GitHubRibbon from "./assets/GitHubRibbon.svg";
import SinglePage from "./components/SinglePage";
import { AuthProvider } from "./context/AuthContext";
import { EnvProvider, useEnv } from "./context/EnvContext";
import { ErrorObject } from "./lib/error";
import KakaoSDK, { isKakaoWebView, kakaoInAppLogin } from "./lib/KakaoSDK";
import "./index.css";

export const links: Route.LinksFunction = () => [
  { rel: "preconnect", href: "https://fonts.googleapis.com" },
  {
    rel: "preconnect",
    href: "https://fonts.gstatic.com",
    crossOrigin: "anonymous",
  },
  {
    rel: "stylesheet",
    href: "https://fonts.googleapis.com/css2?family=Gowun+Batang:wght@400;700&display=swap",
  },
];

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  const env = useEnv();
  const navigate = useNavigate();

  useEffect(() => {
    if (error instanceof ErrorObject) {
      if (error.data.type === "unauthorized") {
        if (isKakaoWebView()) {
          kakaoInAppLogin(env, error.data.redirectTo);
        } else {
          const loginPath = error.data.loginPath ?? "/login/";
          const path = `${loginPath}?redirect_to=${encodeURIComponent(error.data.redirectTo)}`;
          navigate(path);
        }
      }
    }
  }, [env, navigate, error]);

  if (error instanceof ErrorObject) {
    if (error.data.type === "unauthorized") {
      return <></>;
    } else if (error.data.type === "unknown") {
      return (
        <SinglePage>
          <h2>알 수 없는 에러가 발생했습니다.</h2>
          <p>{error.data.error?.message ?? "-"}</p>
        </SinglePage>
      );
    } else if (error.data.type === "remote") {
      return (
        <SinglePage>
          <h2>에러가 발생했습니다.</h2>
          <p>{error.data.message}</p>
        </SinglePage>
      );
    }
  } else if (error instanceof Error) {
    <SinglePage>
      <h2>에러가 발생했습니다.</h2>
      <p>{error.message}</p>
    </SinglePage>;
  } else {
    return <SinglePage>알 수 없는 에러가 발생했습니다.</SinglePage>;
  }
}

export function Layout({ children }: { children: React.ReactNode }) {
  const env = useEnv();

  useEffect(() => {
    ReactGA.initialize(env.gaMeasurementId);
    Modal.setAppElement(document.getElementById("root")!);
  }, [env]);

  return (
    <html lang="ko">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" href="/favicon.png" />
        <link rel="icon" href="/favicon.svg" />
        <KakaoSDK />
        <Meta />
        <Links />
      </head>
      <body id="root">
        {children}
        <div className="bottom-left-ribbon">
          <a
            onClick={() => ReactGA.event("open_github")}
            href="https://github.com/segfault87/dxe"
          >
            <img src={GitHubRibbon} alt="Fork me at GitHub" />
          </a>
        </div>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export default function App() {
  return (
    <EnvProvider>
      <AuthProvider>
        <Outlet />
      </AuthProvider>
    </EnvProvider>
  );
}
