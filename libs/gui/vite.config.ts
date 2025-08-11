import { qwikVite } from '@builder.io/qwik/optimizer';
//import svgx from "@svgx/vite-plugin-qwik";
import tsconfigPaths from 'vite-tsconfig-paths';
import { defineConfig, UserConfig } from 'vite';
import { qwikCity } from "@builder.io/qwik-city/vite";
import { internalIpV4 } from 'internal-ip';
import pkg from "./package.json";


// @ts-expect-error process is a nodejs global
const mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);

const { dependencies = {}, devDependencies = {} } = pkg as any as {
  dependencies: Record<string, string>;
  devDependencies: Record<string, string>;
  [key: string]: unknown;
};


// https://vitejs.dev/config/
export default defineConfig(async ({ command, mode }): Promise<UserConfig> => ({
  plugins: [
    qwikCity(),
    qwikVite({ srcDir: 'src' }),
    //svgx(),
    tsconfigPaths({ root: '.' }),
  ],

  // This tells Vite which dependencies to pre-build in dev mode.
  //optimizeDeps: {
  //  // Put problematic deps that break bundling here, mostly those with binaries.
  //  // For example ['better-sqlite3'] if you use that in server functions.
  //  exclude: [],
  //},

  //build: {
  //  modulePreload: false,
  //},
  // This tells Vite how to bundle the server code.
  //ssr:
  //  command === "build" && mode === "production"
  //    ? {
  //      // All dev dependencies should be bundled in the server build
  //      noExternal: Object.keys(devDependencies),
  //      // Anything marked as a dependency will not be bundled
  //      // These should only be production binary deps (including deps of deps), CLI deps, and their module graph
  //      // If a dep-of-dep needs to be external, add it here
  //      // For example, if something uses `bcrypt` but you don't have it as a dep, you can write
  //      // external: [...Object.keys(dependencies), 'bcrypt']
  //      external: Object.keys(dependencies),
  //    }
  //    : undefined,

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: mobile ? '0.0.0.0' : false,
    hmr: mobile
      ? {
        protocol: 'ws',
        host: await internalIpV4(),
        port: 1421,
      }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}));
