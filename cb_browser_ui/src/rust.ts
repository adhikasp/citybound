// The cargo-web generated glue (target/.../cb_browser_ui.js) is a CommonJS
// module that exports a *promise*, not an object.
//
// Bundlers wrap CommonJS exports in an ES module namespace whose prototype is
// taken from the export, which makes that namespace "look like" a promise
// (a `then` lookup succeeds) without being a real one. Any native promise
// machinery that then adopts it throws
//   "Method Promise.prototype.then called on incompatible receiver #<promise>"
//
// This real ES module keeps the promise intact: importing it always yields a
// genuine promise in `default`.
import * as glue from '../target/wasm32-unknown-unknown/release/cb_browser_ui';

type CBRustAPI = typeof glue.default;

const rustGlue: Promise<CBRustAPI> = (glue as any).default;

export default rustGlue;
