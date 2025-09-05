import tsconfigPaths from 'vite-tsconfig-paths';
import {
  configDefaults,
  defineConfig,
  mergeConfig,
  ViteUserConfig,
} from 'vitest/config';
import viteConfig from './vite.config';
import { resolve } from 'path';

const config = mergeConfig(
  viteConfig, // Extending from an existing Vite configuration (`vite.config.ts` file)
  defineConfig({
    test: {
      ...configDefaults, // Extending Vitest's default options
    },
    plugins: [tsconfigPaths()],
    resolve: {
      alias: [
        {
          find: '@services',
          replacement: resolve(__dirname, './src/services/'),
        },
        { find: '@util', replacement: resolve(__dirname, './src/util/') },
      ],
    },
  }) as ViteUserConfig,
);

export default config;
