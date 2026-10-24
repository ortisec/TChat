import react from "@astrojs/react";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://cli.tchat.site",
  server: { port: 4323 },
  integrations: [react()],
  vite: { plugins: [tailwindcss()] },
});
