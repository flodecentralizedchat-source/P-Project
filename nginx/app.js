async function main() {
  try {
    // wasm-pack outputs files based on crate name:
    // crate: p-project-web -> JS: p_project_web.js, WASM: p_project_web_bg.wasm
    const init = (await import('./p_project_web.js')).default;
    await init();
    console.log('WASM initialized');
    const p = document.querySelector('.muted');
    if (p) p.textContent = 'WASM initialized successfully';
  } catch (err) {
    console.error('Failed to initialize WASM:', err);
    const p = document.querySelector('.muted');
    if (p) p.textContent = 'Failed to initialize WASM. See console.';
  }
}

main();

