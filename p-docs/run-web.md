## Running and Integrating `p-project-web`

1. **Compile for development**:
   ```sh
   cargo build --manifest-path p-project-web/Cargo.toml --target wasm32-unknown-unknown
   ```
   this produces a wasm artifact under `target/wasm32-unknown-unknown/debug`.

2. **Compile a release build or generate a bundler-ready package**:
   ```sh
   cargo build --manifest-path p-project-web/Cargo.toml --target wasm32-unknown-unknown --release
   wasm-pack build --target bundler
   ```
   `wasm-pack` runs `wasm-bindgen`, `wasm-opt`, and emits a JS/wasm package in `p-project-web/pkg` along with a `package.json`.

3. **Use the generated package in your frontend**:
   - Install or link it from your web project (for example `npm install ../p-project-web/pkg` or publish the package).
   - Import and initialize the wasm module:
     ```js
     import init, { WebUser, greet, initialize_app } from "p-project-web";

     async function start() {
       await init();
       const user = new WebUser("id", "name", "wallet");
       console.log(greet("world"));
       initialize_app();
     }

     start();
     ```
   - Ensure your bundler copies `p_project_web_bg.wasm` (webpack asset modules, Rollup copy plugin, or Vite static assets can be used) so the browser can fetch it.

4. **Verify the frontend bundle**:
   - Run `npm run build` (or your equivalent build command) in the frontend project to confirm that the wasm file is bundled and `WebUser`, `greet`, and `initialize_app` are callable from the JS entry point.

Following these steps keeps the wasm crate aligned with your web bundler so the exported helpers remain usable from JavaScript.

Next steps: Install/link `p-project-web/pkg` from your frontend (e.g., `npm install ../p-project-web/pkg`), import the generated module there, and run your bundler’s build command so the wasm outputs end up in the deployed artifact.
