// Bundles src/citybound.tsx into dist/bundle.js.
// (Replaces the parcel-bundler build script.)
const esbuild = require("esbuild");

const watch = process.argv.includes("--watch");

const options = {
    entryPoints: ["src/citybound.tsx"],
    outfile: "dist/bundle.js",
    bundle: true,
    format: "iife",
    target: "es2019",
    platform: "browser",
    // parcel used to babel-transpile everything, including JSX inside .js files
    loader: {
        ".js": "jsx",
        ".png": "dataurl",
        ".svg": "dataurl",
        ".jpg": "dataurl",
        ".woff": "dataurl",
        ".woff2": "dataurl",
    },
    // only reached in the node branch of the cargo-web glue
    external: ["fs", "path"],
    define: {
        "process.env.NODE_ENV": watch ? '"development"' : '"production"',
    },
    logLevel: "info",
};

(async () => {
    if (watch) {
        const ctx = await esbuild.context(options);
        await ctx.watch();
        console.log("watching src/ ...");
    } else {
        await esbuild.build(options);
    }
})().catch(err => {
    console.error(err);
    process.exit(1);
});
