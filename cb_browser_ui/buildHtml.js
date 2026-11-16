// Assembles dist/ for the browser UI: copies index.html with its asset
// references pointed at the bundled files, and copies static assets.
// (Replaces the HTML post-processing that parcel-bundler used to do.)
const fs = require("fs");
const path = require("path");

const dist = "dist";
fs.existsSync(dist) || fs.mkdirSync(dist, { recursive: true });

const html = fs
    .readFileSync("index.html", "utf8")
    .replace('href="main.less"', 'href="main.css"')
    .replace('src="./src/citybound.tsx"', 'src="./bundle.js"');

fs.writeFileSync(path.join(dist, "index.html"), html);

const assets = "assets";
if (fs.existsSync(assets)) {
    fs.cpSync(assets, path.join(dist, "assets"), { recursive: true });
}

console.log("dist/index.html written");
