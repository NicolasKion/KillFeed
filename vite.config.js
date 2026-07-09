import inertia from '@inertiajs/vite';
import { wayfinder } from '@laravel/vite-plugin-wayfinder';
import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import laravel from 'laravel-vite-plugin';
import { fileURLToPath } from 'node:url';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
    const env = loadEnv(mode, process.cwd(), '');

    return {
        resolve: {
            alias: {
                '@': fileURLToPath(new URL('./resources/js', import.meta.url)),
            },
        },
        plugins: [
            wayfinder(),
            tailwindcss(),
            laravel({
                input: 'resources/js/app.ts',
                refresh: true,
            }),
            inertia({
                ssr: {
                    port: Number(env.VITE_INERTIA_SSR_PORT ?? 13714),
                },
            }),
            vue({
                template: {
                    transformAssetUrls: {
                        base: null,
                        includeAbsolute: false,
                    },
                },
            }),
        ],
    };
});
