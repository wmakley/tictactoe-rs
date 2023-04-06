import { building } from '$app/environment';
import type { LayoutLoad } from './$types';

export const prerender = true;

export const load = (() => {
    return {
        socketUrl: (building ? '/' : 'localhost:3000/')
    };
}) satisfies LayoutLoad;
