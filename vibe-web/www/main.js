import init, { VibeApp } from '../pkg/vibe_web.js';

function showError(msg) {
    let el = document.getElementById('error-overlay');
    if (!el) {
        el = document.createElement('div');
        el.id = 'error-overlay';
        el.style.cssText = 'position:fixed;top:10px;left:10px;right:10px;background:rgba(200,0,0,0.9);color:#fff;padding:16px;font:14px monospace;z-index:9999;white-space:pre-wrap;border-radius:8px;max-height:50vh;overflow:auto;';
        document.body.appendChild(el);
    }
    el.textContent += msg + '\n';
    console.error('[vibe]', msg);
}

function showStatus(msg) {
    let el = document.getElementById('status-overlay');
    if (!el) {
        el = document.createElement('div');
        el.id = 'status-overlay';
        el.style.cssText = 'position:fixed;bottom:10px;left:10px;background:rgba(0,0,0,0.7);color:#0f0;padding:8px 12px;font:12px monospace;z-index:9999;border-radius:4px;';
        document.body.appendChild(el);
    }
    el.textContent = msg;
    console.log('[vibe]', msg);
}

async function main() {
    showStatus('Starting...');

    if (!navigator.gpu) {
        showError('WebGPU not supported in this browser');
        document.getElementById('no-webgpu').style.display = 'flex';
        return;
    }
    showStatus('WebGPU available, initializing WASM...');

    await init();
    showStatus('WASM loaded, creating VibeApp...');

    const canvas = document.getElementById('vibe-canvas');
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    // Create app (async constructor — must await)
    const app = await new VibeApp('vibe-canvas');
    showStatus(`VibeApp created (${canvas.width}x${canvas.height}), rendering fallback...`);

    app.resize(canvas.width, canvas.height);

    // First test: render a few frames with the FALLBACK shader to verify pipeline works
    for (let i = 0; i < 5; i++) {
        app.render();
    }
    showStatus('Fallback rendered OK, loading custom shader...');

    // Now load and apply the custom shader
    const resp = await fetch('shaders/default.wgsl');
    const shaderCode = await resp.text();
    app.set_shader(shaderCode);
    showStatus(`Shader loaded (${shaderCode.length} chars), rendering...`);

    window.addEventListener('resize', () => {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        app.resize(canvas.width, canvas.height);
    });

    canvas.addEventListener('mousemove', (e) => {
        app.set_mouse(e.clientX / window.innerWidth, e.clientY / window.innerHeight);
    });
    canvas.addEventListener('click', (e) => {
        app.on_click(e.clientX / window.innerWidth, e.clientY / window.innerHeight);
    });

    // Render loop
    let frameCount = 0;
    function frame() {
        try {
            app.render();
            frameCount++;
            if (frameCount === 1) showStatus('First frame rendered');
            if (frameCount === 60) {
                // Remove status overlay after ~1s of successful rendering
                const statusEl = document.getElementById('status-overlay');
                if (statusEl) statusEl.remove();
            }
        } catch (e) {
            showError(`Render error (frame ${frameCount}): ${e}`);
        }
        requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
}

main().catch(e => showError(`Fatal: ${e}`));
