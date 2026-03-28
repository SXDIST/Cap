import { fileURLToPath } from "node:url";
import capUIPlugin from "@cap/ui-solid/vite";
import { defineConfig } from "@solidjs/start/config";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
	ssr: false,
	server: { preset: "static" },
	vite: () => ({
		server: {
			host: "localhost",
			port: 3210,
			strictPort: true,
			watch: {
				ignored: ["**/src-tauri/**"],
			},
			headers: {
				"Cross-Origin-Opener-Policy": "same-origin",
				"Cross-Origin-Embedder-Policy": "require-corp",
			},
		},
		envPrefix: ["VITE_", "TAURI_"],
		assetsInclude: ["**/*.riv"],
		resolve: {
			alias: {
				debug: fileURLToPath(new URL("./src/shims/debug-browser.ts", import.meta.url)),
				extend: fileURLToPath(new URL("./src/shims/extend.ts", import.meta.url)),
			},
		},
		plugins: [
			wasm(),
			topLevelAwait(),
			capUIPlugin,
			tsconfigPaths({
				root: ".",
			}),
		],
		define: {
			"import.meta.vitest": "undefined",
		},
		optimizeDeps: {
			include: [
				"@tauri-apps/plugin-os",
				"@tanstack/solid-query",
				"@tauri-apps/api/webviewWindow",
				"@tauri-apps/plugin-dialog",
				"@tauri-apps/plugin-store",
				"posthog-js",
				"uuid",
				"@tauri-apps/plugin-clipboard-manager",
				"@tauri-apps/api/window",
				"@tauri-apps/api/core",
				"@tauri-apps/api/event",
				"cva",
				"extend",
			],
		},
	}),
});
