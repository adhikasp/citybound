const fs = require("fs");
fs.existsSync("dist") || fs.mkdirSync("dist");
fs.copyFileSync("target/wasm-bindgen/cb_browser_ui_bg.wasm", "dist/cb_browser_ui_bg.wasm");
