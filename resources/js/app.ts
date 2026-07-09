import '../css/app.css';

import { createInertiaApp } from '@inertiajs/vue3';
import { resolvePageComponent } from 'laravel-vite-plugin/inertia-helpers';
import { DefineComponent } from 'vue';

const appName = import.meta.env.VITE_APP_NAME || 'KillFeed';

createInertiaApp({
    title: (title) => `${title} - ${appName}`,
    resolve: (name) => resolvePageComponent(`./Pages/${name}.vue`, import.meta.glob<DefineComponent>('./Pages/**/*.vue')),
    progress: {
        color: '#e5484d',
        delay: 1000,
    },
});
