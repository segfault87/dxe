import type { Config } from "@react-router/dev/config";

export default {
  async prerender() {
    return ["/", "/guide", "/inquiries", "reservation"];
  },
  routeDiscovery: { mode: "initial" },
} satisfies Config;
