import react from "@astrojs/react";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://tchat.site",
  server: { port: 4321 },
  integrations: [react()],
  vite: { plugins: [tailwindcss()] },
});
