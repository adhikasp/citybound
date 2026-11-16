// Loads the wasm-bindgen output and publishes the JS host functions the
// simulation calls. The default export resolves to the wasm API, which
// citybound.tsx installs as window.cbRustBrowser and calls start() on.
import init, * as wasm from '../target/wasm-bindgen/cb_browser_ui.js';
import * as host from './host.js';

(window as any).cbHost = host;

const ready = (init as any)('cb_browser_ui_bg.wasm').then(() => wasm as any);

export default ready;
