import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
// `defineConfig` from vitest/config understands the extra `test` field while
// still being a drop-in for vite's own defineConfig.
import { defineConfig } from "vitest/config";
import { fileURLToPath } from "node:url";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 5174 }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/vendor/**"],
    },
  },
  test: {
    environment: "jsdom",
    globals: false,
    include: ["src/**/*.{test,spec}.ts"],
    // SvelteKit's `$app/environment` only resolves inside a real app build, so
    // tests get a tiny stub instead. `browser=false` keeps store constructors
    // from touching jsdom's localStorage by default — flip per-test if needed.
    alias: {
      "$app/environment": fileURLToPath(
        new URL("./test/mocks/app-environment.ts", import.meta.url),
      ),
    },
  },
});
