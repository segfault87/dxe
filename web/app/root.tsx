import React, { useEffect } from "react";
import ReactGA from "react-ga4";
import Modal from "react-modal";
import { Outlet, Links, Meta, Scripts, ScrollRestoration } from "react-router";

import GitHubRibbon from "./assets/GitHubRibbon.svg";
import { AuthProvider } from "./context/AuthContext";
import { EnvProvider, useEnv } from "./context/EnvContext";
import KakaoSDK from "./lib/KakaoSDK";
import type { Route } from "./+types/root";
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
