// EGrab - Frontend Entry Point
// Mounts the Svelte 5 application to the DOM

import './app.css';
import App from './App.svelte';
import { mount } from 'svelte';

const app = mount(App, { target: document.getElementById('app')! });

export default app;
