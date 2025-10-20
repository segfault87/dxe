import React, { useEffect } from "react";
import { Outlet, Links, Meta, Scripts, ScrollRestoration } from "react-router";
import TagManager from "react-gtm-module";
import ReactGA from "react-ga4";

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
  return (
    <html lang="ko">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body>
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export default function App() {
  useEffect(() => {
    TagManager.initialize({ gtmId: "GTM-K2MNLT2P" });
    ReactGA.initialize("G-2RQMYRGB4Q");
  }, []);

  return <Outlet />;
}
