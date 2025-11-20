import type { Config } from "@react-router/dev/config";

export default {
  ssr: false,
  async prerender() {
    return ["/", "/guide", "/inquiries", "/terms-of-service"];
  },
  routeDiscovery: { mode: "initial" },
} satisfies Config;
