import { defineConfig } from "vite";
import { qwikVite } from "@builder.io/qwik/optimizer";
import { qwikCity } from "@builder.io/qwik-city/vite";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig(() => ({
  plugins: [
    qwikCity({
      routesDir: 'src-frontend/routes',
    }),
    qwikVite({
      srcDir: 'src-frontend',
    }),
    tsconfigPaths()
  ],
  resolve: {
    alias: {
      //  '@': './src-frontend',
    },
  },
  dev: {
    headers: {
      "Cache-Control": "public, max-age=0",
    },
  },
  preview: {
    headers: {
      "Cache-Control": "public, max-age=600",
    },
  },
  build: {
    ssr: false,  // Ensure SSR is turned off for CSR build
  },
})
                           );
