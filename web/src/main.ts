import './app.css';
import App from './App.svelte';
import { mount } from 'svelte';
import { initTheme } from './lib/theme.ts';

initTheme();

const app = mount(App, { target: document.getElementById('app')! });

export default app;
