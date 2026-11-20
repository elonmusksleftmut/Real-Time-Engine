<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Higher-Order Spectral &amp; Time-Arrow Analyzer (WebGL 2.0)</title>
  <style>
    /* =========================================================================
       Precision Studio-Instrument Theme (Zero External Dependencies)
       ========================================================================= */
    :root {
      --bg-0: #06090f;
      --bg-1: #0c121d;
      --bg-2: #121b2b;
      --bg-3: #19253a;
      --border: #23324d;
      --border-hi: #354c73;
      --text-0: #e8eef7;
      --text-1: #9fb0c9;
      --text-2: #647896;
      --cyan: #22d3ee;
      --indigo: #6366f1;
      --amber: #f59e0b;
      --crimson: #ef4444;
      --emerald: #10b981;
      --violet: #a855f7;
      --mono: ui-monospace, "SFMono-Regular", Menlo, Monaco, Consolas, "Liberation Mono", monospace;
      --sans: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      --radius: 10px;
    }

    *, *::before, *::after { box-sizing: border-box; }

    html, body {
      margin: 0;
      padding: 0;
      background: var(--bg-0);
      color: var(--text-0);
      font-family: var(--sans);
      font-size: 14px;
      line-height: 1.4;
      overflow-x: hidden;
      -webkit-font-smoothing: antialiased;
    }

    .app {
      max-width: 1440px;
      margin: 0 auto;
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 12px;
    }

    /* Header Bar */
    .header {
      background: linear-gradient(180deg, var(--bg-2), var(--bg-1));
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 12px 16px;
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
    }

    .brand {
      display: flex;
      flex-direction: column;
      gap: 2px;
    }

    .brand-title {
      font-size: 16px;
      font-weight: 700;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .brand-dot {
      width: 9px;
      height: 9px;
      border-radius: 50%;
      background: var(--text-2);
      box-shadow: 0 0 8px transparent;
      transition: background 0.2s, box-shadow 0.2s;
    }
    .brand-dot.live {
      background: var(--emerald);
      box-shadow: 0 0 10px var(--emerald);
    }
    .brand-dot.gated {
      background: var(--amber);
      box-shadow: 0 0 10px var(--amber);
    }

    .brand-sub {
      font-family: var(--mono);
      font-size: 11px;
      color: var(--text-2);
    }

    .header-right {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 10px;
    }

    .verdict-badge {
      font-family: var(--mono);
      font-weight: 700;
      font-size: 12px;
      letter-spacing: 0.06em;
      padding: 7px 14px;
      border-radius: 999px;
      border: 1px solid var(--border-hi);
      background: var(--bg-3);
      color: var(--text-1);
      text-transform: uppercase;
      transition: all 0.18s ease;
    }
    .verdict-badge.v-clean   { border-color: #334155; color: #94a3b8; background: #0f172a; }
    .verdict-badge.v-even    { border-color: var(--amber); color: #fde68a; background: rgba(245,158,11,0.16); box-shadow: 0 0 14px rgba(245,158,11,0.22); }
    .verdict-badge.v-odd     { border-color: var(--violet); color: #e9d5ff; background: rgba(168,85,247,0.18); box-shadow: 0 0 14px rgba(168,85,247,0.25); }
    .verdict-badge.v-causal  { border-color: var(--emerald); color: #a7f3d0; background: rgba(16,185,129,0.16); box-shadow: 0 0 14px rgba(16,185,129,0.24); }
    .verdict-badge.v-reverse { border-color: var(--crimson); color: #fecaca; background: rgba(239,68,68,0.18); box-shadow: 0 0 14px rgba(239,68,68,0.26); }
    .verdict-badge.v-smeared { border-color: var(--cyan); color: #cffafe; background: rgba(34,211,238,0.16); box-shadow: 0 0 14px rgba(34,211,238,0.22); }

    .sys-pill {
      font-family: var(--mono);
      font-size: 11px;
      color: var(--text-1);
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 6px 10px;
    }

    /* Status / Error Banner */
    .status-banner {
      display: none;
      font-family: var(--mono);
      font-size: 12px;
      padding: 9px 14px;
      border-radius: 8px;
      border: 1px solid var(--amber);
      background: rgba(245, 158, 11, 0.12);
      color: #fde68a;
    }
    .status-banner.error {
      border-color: var(--crimson);
      background: rgba(239, 68, 68, 0.14);
      color: #fecaca;
    }
    .status-banner.show { display: block; }

    /* Cards & Panels */
    .panel {
      background: var(--bg-1);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 10px;
    }

    .panel-head {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: 8px;
      border-bottom: 1px solid var(--border);
      padding-bottom: 8px;
    }

    .panel-title {
      font-family: var(--mono);
      font-size: 12px;
      font-weight: 700;
      letter-spacing: 0.05em;
      text-transform: uppercase;
      color: var(--text-0);
    }

    .panel-meta {
      font-family: var(--mono);
      font-size: 11px;
      color: var(--text-2);
    }

    /* Controls Grid */
    .controls-layout {
      display: grid;
      grid-template-columns: 1.45fr 1fr;
      gap: 12px;
    }

    .tab-row {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
    }

    button {
      font-family: var(--mono);
      font-size: 12px;
      font-weight: 600;
      color: var(--text-0);
      background: var(--bg-2);
      border: 1px solid var(--border);
      border-radius: 7px;
      padding: 8px 12px;
      min-height: 36px;
      cursor: pointer;
      transition: background 0.15s, border-color 0.15s, color 0.15s, transform 0.08s;
      user-select: none;
    }
    button:hover {
      background: var(--bg-3);
      border-color: var(--border-hi);
    }
    button:active { transform: translateY(1px); }

    button.active {
      background: rgba(34, 211, 238, 0.16);
      border-color: var(--cyan);
      color: #e0f2fe;
    }

    button.primary {
      background: rgba(16, 185, 129, 0.18);
      border-color: var(--emerald);
      color: #d1fae5;
    }
    button.primary:hover { background: rgba(16, 185, 129, 0.28); }

    button.danger {
      background: rgba(239, 68, 68, 0.18);
      border-color: var(--crimson);
      color: #fee2e2;
    }
    button.danger:hover { background: rgba(239, 68, 68, 0.28); }

    button.warn-toggle.active {
      background: rgba(245, 158, 11, 0.2);
      border-color: var(--amber);
      color: #fef3c7;
    }

    .source-pane { display: none; flex-direction: column; gap: 10px; }
    .source-pane.active { display: flex; }

    .scene-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(175px, 1fr));
      gap: 6px;
    }

    .scene-btn {
      text-align: left;
      padding: 8px 10px;
      font-size: 11.5px;
      line-height: 1.25;
      display: flex;
      flex-direction: column;
      gap: 2px;
    }
    .scene-btn small {
      font-weight: 400;
      font-size: 10px;
      color: var(--text-2);
    }
    .scene-btn.active small { color: #7dd3fc; }

    .inline-row {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 10px;
    }

    /* Sliders */
    .slider-group {
      display: flex;
      flex-direction: column;
      gap: 4px;
      min-width: 160px;
      flex: 1;
    }
    .slider-label {
      display: flex;
      justify-content: space-between;
      font-family: var(--mono);
      font-size: 11px;
      color: var(--text-1);
    }
    .slider-label span:last-child { color: var(--cyan); font-weight: 700; }

    input[type="range"] {
      -webkit-appearance: none;
      appearance: none;
      width: 100%;
      height: 6px;
      border-radius: 999px;
      background: var(--bg-3);
      outline: none;
      cursor: pointer;
    }
    input[type="range"]::-webkit-slider-thumb {
      -webkit-appearance: none;
      appearance: none;
      width: 16px;
      height: 16px;
      border-radius: 50%;
      background: var(--cyan);
      border: 2px solid var(--bg-0);
      box-shadow: 0 0 6px rgba(34, 211, 238, 0.5);
    }
    input[type="range"]::-moz-range-thumb {
      width: 16px;
      height: 16px;
      border-radius: 50%;
      background: var(--cyan);
      border: 2px solid var(--bg-0);
    }

    /* Dropzone */
    .dropzone {
      border: 1.5px dashed var(--border-hi);
      border-radius: 8px;
      background: var(--bg-0);
      padding: 14px;
      text-align: center;
      cursor: pointer;
      transition: border-color 0.15s, background 0.15s;
      font-family: var(--mono);
      font-size: 12px;
      color: var(--text-1);
    }
    .dropzone.dragover {
      border-color: var(--cyan);
      background: rgba(34, 211, 238, 0.08);
      color: var(--text-0);
    }
    .file-info {
      font-family: var(--mono);
      font-size: 11px;
      color: var(--text-2);
      word-break: break-all;
    }

    /* WebGL Heatmap Grid */
    .heatmaps-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 12px;
    }

    .gl-wrap {
      position: relative;
      width: 100%;
      aspect-ratio: 16 / 10;
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 8px;
      overflow: hidden;
    }

    .gl-canvas {
      display: block;
      width: 100%;
      height: 100%;
      touch-action: none;
      cursor: crosshair;
    }

    .axis-y-label {
      position: absolute;
      left: 6px;
      top: 6px;
      font-family: var(--mono);
      font-size: 10px;
      color: rgba(232, 238, 247, 0.72);
      background: rgba(6, 9, 15, 0.72);
      padding: 2px 6px;
      border-radius: 4px;
      pointer-events: none;
    }
    .axis-x-label {
      position: absolute;
      right: 6px;
      bottom: 6px;
      font-family: var(--mono);
      font-size: 10px;
      color: rgba(232, 238, 247, 0.72);
      background: rgba(6, 9, 15, 0.72);
      padding: 2px 6px;
      border-radius: 4px;
      pointer-events: none;
    }
    .gl-tag {
      position: absolute;
      right: 6px;
      top: 6px;
      font-family: var(--mono);
      font-size: 10px;
      color: var(--cyan);
      background: rgba(6, 9, 15, 0.78);
      border: 1px solid rgba(34, 211, 238, 0.3);
      padding: 2px 6px;
      border-radius: 4px;
      pointer-events: none;
    }

    .inspector-bar {
      font-family: var(--mono);
      font-size: 11px;
      color: var(--text-1);
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 6px 10px;
      display: flex;
      flex-wrap: wrap;
      justify-content: space-between;
      gap: 6px;
      min-height: 30px;
      align-items: center;
    }
    .inspector-bar strong { color: var(--text-0); }
    .sig-pill {
      padding: 1px 6px;
      border-radius: 4px;
      font-weight: 700;
      font-size: 10px;
    }
    .sig-pill.yes { background: rgba(239, 68, 68, 0.25); color: #fca5a5; border: 1px solid var(--crimson); }
    .sig-pill.no  { background: rgba(100, 116, 139, 0.2); color: var(--text-2); border: 1px solid var(--border); }

    /* Telemetry HUD */
    .hud-grid {
      display: grid;
      grid-template-columns: repeat(6, 1fr);
      gap: 8px;
    }

    .metric-card {
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 9px 10px;
      display: flex;
      flex-direction: column;
      gap: 4px;
    }

    .metric-label {
      font-family: var(--mono);
      font-size: 10px;
      color: var(--text-2);
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }

    .metric-val {
      font-family: var(--mono);
      font-size: 16px;
      font-weight: 700;
      color: var(--text-0);
    }

    .metric-sub {
      font-family: var(--mono);
      font-size: 10px;
      color: var(--text-2);
    }

    /* Bidirectional phi_rev bar */
    .bidir-track {
      position: relative;
      height: 8px;
      background: var(--bg-2);
      border-radius: 999px;
      overflow: hidden;
      margin-top: 2px;
    }
    .bidir-center {
      position: absolute;
      left: 50%;
      top: 0;
      bottom: 0;
      width: 1px;
      background: var(--text-2);
      z-index: 2;
    }
    .bidir-fill {
      position: absolute;
      top: 0;
      bottom: 0;
      width: 0%;
      left: 50%;
      background: var(--emerald);
      transition: left 0.08s linear, width 0.08s linear, background 0.12s;
    }

    /* Oscilloscope + RTA Strip */
    .scope-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 12px;
    }
    .scope-wrap {
      position: relative;
      width: 100%;
      height: 155px;
      background: var(--bg-0);
      border: 1px solid var(--border);
      border-radius: 8px;
      overflow: hidden;
    }
    .scope-canvas {
      display: block;
      width: 100%;
      height: 100%;
    }

    .colormap-bar {
      display: flex;
      align-items: center;
      gap: 8px;
      font-family: var(--mono);
      font-size: 10px;
      color: var(--text-2);
    }
    .colormap-grad {
      height: 8px;
      flex: 1;
      border-radius: 4px;
      background: linear-gradient(90deg, #070b14 0%, #1c2463 24%, #0ca5c8 50%, #f59e0b 75%, #ef4444 90%, #fff7ed 100%);
      border: 1px solid var(--border);
    }

    /* Responsive Breakpoints */
    @media (max-width: 1080px) {
      .controls-layout { grid-template-columns: 1fr; }
      .hud-grid { grid-template-columns: repeat(3, 1fr); }
    }

    @media (max-width: 820px) {
      .heatmaps-grid { grid-template-columns: 1fr; }
      .scope-grid { grid-template-columns: 1fr; }
      .hud-grid { grid-template-columns: repeat(2, 1fr); }
      .app { padding: 8px; }
    }
  </style>
</head>
<body>
  <div class="app">
    <!-- =====================================================================
         TOP HEADER & LIVE VERDICT HUD
         ===================================================================== -->
    <header class="header">
      <div class="brand">
        <div class="brand-title">
          <span id="liveDot" class="brand-dot"></span>
          <span>Higher-Order Spectral &amp; Time-Arrow Analyzer</span>
        </div>
        <div class="brand-sub">
          Rolling EMA Bicoherence (3rd) &middot; Tricoherence (4th) &middot; Causal Bandt-Pompe D_JS &middot; WebGL 2.0
        </div>
      </div>
      <div class="header-right">
        <div id="verdictBadge" class="verdict-badge v-clean">IDLE / STANDBY</div>
        <div id="sysTelemetry" class="sys-pill">fs_a: -- Hz | Tap: -- | Hop: -- ms</div>
      </div>
    </header>

    <div id="statusBanner" class="status-banner"></div>

    <!-- =====================================================================
         INPUT SOURCES & DSP ENGINE CONTROLS
         ===================================================================== -->
    <section class="controls-layout">
      <!-- Left: Audio Source Selector -->
      <div class="panel">
        <div class="panel-head">
          <div class="panel-title">Audio Input Source</div>
          <div class="tab-row">
            <button id="tabSynth" class="active" type="button">1. Synth Testbed</button>
            <button id="tabMic" type="button">2. Live Mic</button>
            <button id="tabFile" type="button">3. Audio File</button>
          </div>
        </div>

        <!-- Source Pane 1: Synth Testbed -->
        <div id="paneSynth" class="source-pane active">
          <div class="scene-grid" id="sceneGrid">
            <button class="scene-btn active" data-scene="0" type="button">
              1. Clean Uncoupled Mix
              <small>62.5 + 281.25 + 343.75 Hz rand &phi;</small>
            </button>
            <button class="scene-btn" data-scene="1" type="button">
              2. Odd Clipper (tanh x&sup3;)
              <small>Symmetric clip &rarr; 4th-order spike</small>
            </button>
            <button class="scene-btn" data-scene="2" type="button">
              3. Tube + Sub IMD (x&sup2;)
              <small>Asym shaper &rarr; 3rd-order IMD</small>
            </button>
            <button class="scene-btn" data-scene="3" type="button">
              4. Dry Causal Kick
              <small>125 BPM saw kick &rarr; +forward arrow</small>
            </button>
            <button class="scene-btn" data-scene="4" type="button">
              5. All-Pass Smeared Kick
              <small>10-stage |H|=1 &rarr; erased arrow</small>
            </button>
            <button class="scene-btn" data-scene="5" type="button">
              6. Literal Reversed Kick
              <small>Time-flipped &rarr; &minus;&phi;_rev &amp; &minus;Im[B]</small>
            </button>
            <button class="scene-btn" data-scene="6" type="button">
              7. Linear-Phase Pre-Ring
              <small>Zero-phase 420Hz &rarr; pre-onset ring</small>
            </button>
          </div>
          <div class="inline-row">
            <button id="btnSynthToggle" class="primary" type="button">&#9654; Start Synth</button>
            <div class="slider-group">
              <div class="slider-label">
                <span>Monitor Volume (Speakers)</span>
                <span id="lblVolume">35%</span>
              </div>
              <input id="slVolume" type="range" min="0" max="1" step="0.01" value="0.35" />
            </div>
          </div>
        </div>

        <!-- Source Pane 2: Live Microphone -->
        <div id="paneMic" class="source-pane">
          <div class="inline-row">
            <button id="btnMicToggle" class="primary" type="button">&#9679; Start Live Mic</button>
            <span class="panel-meta">
              Raw DSP mode (echo/NS/AGC disabled). Routed strictly to analyzer &mdash; never to speakers.
            </span>
          </div>
        </div>

        <!-- Source Pane 3: Drag-and-Drop File Player -->
        <div id="paneFile" class="source-pane">
          <div id="dropzone" class="dropzone">
            Drop .wav, .mp3, .ogg, .m4a, or .flac here &mdash; or tap to browse
            <input id="fileInput" type="file" accept="audio/*,.wav,.mp3,.ogg,.m4a,.flac" style="display:none" />
          </div>
          <div id="fileInfo" class="file-info">No audio file loaded.</div>
          <div class="inline-row">
            <button id="btnFilePlay" class="primary" type="button" disabled>&#9654; Play</button>
            <button id="btnFileLoop" class="active" type="button">Loop: ON</button>
            <div class="slider-group">
              <div class="slider-label">
                <span>Seek Position</span>
                <span id="lblFileTime">0:00 / 0:00</span>
              </div>
              <input id="slFileSeek" type="range" min="0" max="1" step="0.001" value="0" disabled />
            </div>
          </div>
          <div class="slider-group">
            <div class="slider-label">
              <span>File Monitor Volume</span>
              <span id="lblFileVolume">35%</span>
            </div>
            <input id="slFileVolume" type="range" min="0" max="1" step="0.01" value="0.35" />
          </div>
        </div>
      </div>

      <!-- Right: Rolling EMA & GPU Shader Controls -->
      <div class="panel">
        <div class="panel-head">
          <div class="panel-title">Rolling EMA &amp; GPU Shader Controls</div>
          <div class="panel-meta" id="lblKeff">K_eff(3): 0.0 | K_eff(4): 0.0</div>
        </div>

        <div class="slider-group">
          <div class="slider-label">
            <span>EMA Smoothing Time (&tau;_s)</span>
            <span id="lblTau">0.60 s</span>
          </div>
          <input id="slTau" type="range" min="0.2" max="2.5" step="0.05" value="0.60" />
        </div>

        <div class="slider-group">
          <div class="slider-label">
            <span>Silence Gate (Freezes EMA)</span>
            <span id="lblGate">-55 dBFS</span>
          </div>
          <input id="slGate" type="range" min="-70" max="-30" step="1" value="-55" />
        </div>

        <div class="inline-row">
          <button id="btnAsymShader" class="warn-toggle" type="button">
            Im[B] Asymmetry Shader: OFF
          </button>
          <button id="btnResetEma" type="button">Reset EMA</button>
        </div>

        <div class="colormap-bar">
          <span>0.0</span>
          <div class="colormap-grad"></div>
          <span>1.0</span>
          <span style="color:#fca5a5;">[Bright Contour = Bonf p&lt;0.05]</span>
        </div>
      </div>
    </section>

    <!-- =====================================================================
         DUAL WEBGL 2.0 HIGHER-ORDER GPU HEATMAPS
         ===================================================================== -->
    <section class="heatmaps-grid">
      <!-- 3rd-Order Bicoherence Panel -->
      <div class="panel">
        <div class="panel-head">
          <div class="panel-title">3rd-Order Bicoherence b&sup2;(f&#8321;, f&#8322;) &mdash; Even / IMD</div>
          <div id="metaBi" class="panel-meta">Bonf crit: -- | Hits: 0 / 1024</div>
        </div>
        <div class="gl-wrap">
          <canvas id="glBi" class="gl-canvas"></canvas>
          <div class="axis-y-label" id="lblBiY">f&#8322; &uarr; 0 .. 1000 Hz</div>
          <div class="axis-x-label" id="lblBiX">f&#8321; &rarr; 0 .. 2000 Hz (f&#8321;+f&#8322; &le; 64 bins)</div>
          <div class="gl-tag" id="tagBiGL">WebGL 2.0</div>
        </div>
        <div id="inspBi" class="inspector-bar">
          <span>Hover/touch heatmap to inspect exact triad (f&#8321;, f&#8322; &rarr; f&#8323;)</span>
          <span id="topBiSummary">Top: --</span>
        </div>
      </div>

      <!-- 4th-Order Tricoherence Panel -->
      <div class="panel">
        <div class="panel-head">
          <div class="panel-title">4th-Order Tricoherence max_f&#8323; t&sup2;(f&#8321;, f&#8322;, f&#8323;) &mdash; Odd Clip</div>
          <div id="metaTri" class="panel-meta">Bonf crit: -- | Hits: 0 / --</div>
        </div>
        <div class="gl-wrap">
          <canvas id="glTri" class="gl-canvas"></canvas>
          <div class="axis-y-label" id="lblTriY">f&#8322; &uarr; 0 .. 560 Hz</div>
          <div class="axis-x-label" id="lblTriX">f&#8321; &rarr; 0 .. 1125 Hz (f&#8321;+f&#8322;+f&#8323; &le; 36 bins)</div>
          <div class="gl-tag" id="tagTriGL">WebGL 2.0</div>
        </div>
        <div id="inspTri" class="inspector-bar">
          <span>Hover/touch heatmap to inspect winning quartet (f&#8321;, f&#8322;, f&#8323; &rarr; f&#8324;)</span>
          <span id="topTriSummary">Top: --</span>
        </div>
      </div>
    </section>

    <!-- =====================================================================
         CAUSAL TIME-ARROW & SPECTRAL TELEMETRY HUD
         ===================================================================== -->
    <section class="hud-grid">
      <div class="metric-card">
        <div class="metric-label">Increment Skew &phi;_rev</div>
        <div id="valPhiRev" class="metric-val">+0.000</div>
        <div class="bidir-track">
          <div class="bidir-center"></div>
          <div id="barPhiRev" class="bidir-fill"></div>
        </div>
        <div class="metric-sub">&minus;Reverse &larr; 0 &rarr; +Causal</div>
      </div>

      <div class="metric-card">
        <div class="metric-label">Ordinal Irrev D_JS (D=3)</div>
        <div id="valDjs" class="metric-val">0.0000</div>
        <div class="metric-sub">nats (64-bin REV_LUT)</div>
      </div>

      <div class="metric-card">
        <div class="metric-label">Imaginary Bispec Im_RMS</div>
        <div id="valImRms" class="metric-val">0.0000</div>
        <div id="subImBias" class="metric-sub">Polarity: +0.000</div>
      </div>

      <div class="metric-card">
        <div class="metric-label">Peak Crest Factor</div>
        <div id="valCrest" class="metric-val">0.00</div>
        <div id="subCrestDb" class="metric-sub">0.0 dB (Peak/RMS)</div>
      </div>

      <div class="metric-card">
        <div class="metric-label">Spectral Entropy H_spec</div>
        <div id="valHspec" class="metric-val">0.000</div>
        <div class="metric-sub">0.0 = pure tone, 1.0 = white</div>
      </div>

      <div class="metric-card">
        <div class="metric-label">Active Peak Coupling</div>
        <div id="valPeaks" class="metric-val">0.00 / 0.00</div>
        <div id="subPeaks" class="metric-sub">b&sup2; max / t&sup2; max</div>
      </div>
    </section>

    <!-- =====================================================================
         SYNCHRONIZED 2D OSCILLOSCOPE & 2ND-ORDER RTA POWER SPECTRUM
         ===================================================================== -->
    <section class="scope-grid">
      <div class="panel">
        <div class="panel-head">
          <div class="panel-title">Peak-Triggered Oscilloscope (Decimated Stream)</div>
          <div id="metaScope" class="panel-meta">Pre-onset left &middot; Decay/Chirp right</div>
        </div>
        <div class="scope-wrap">
          <canvas id="cvScope" class="scope-canvas"></canvas>
        </div>
      </div>

      <div class="panel">
        <div class="panel-head">
          <div class="panel-title">2nd-Order RTA Power Spectrum (Phase-Blind)</div>
          <div class="inline-row" style="gap:6px;">
            <button id="btnLockRta" type="button" style="min-height:26px;padding:3px 9px;font-size:11px;">
              Lock Ref Trace
            </button>
            <span id="metaRta" class="panel-meta">0 .. 4000 Hz</span>
          </div>
        </div>
        <div class="scope-wrap">
          <canvas id="cvRta" class="scope-canvas"></canvas>
        </div>
      </div>
    </section>
  </div>

  <script>
  (() => {
    "use strict";

    // =========================================================================
    // 1. EXACT IN-PLACE RADIX-2 COOLEY-TUKEY FFT (Zero External Dependencies)
    // =========================================================================
    class Radix2FFT {
      constructor(size) {
        if ((size & (size - 1)) !== 0 || size < 2) {
          throw new Error("FFT size must be a power of 2 >= 2");
        }
        this.size = size;
        this.bitRev = new Uint32Array(size);
        const bits = Math.round(Math.log2(size));
        for (let i = 0; i < size; i++) {
          let rev = 0, x = i;
          for (let b = 0; b < bits; b++) {
            rev = (rev << 1) | (x & 1);
            x >>= 1;
          }
          this.bitRev[i] = rev;
        }
        const half = size >> 1;
        this.cosTable = new Float64Array(half);
        this.sinTable = new Float64Array(half);
        for (let i = 0; i < half; i++) {
          const ang = -2.0 * Math.PI * i / size;
          this.cosTable[i] = Math.cos(ang);
          this.sinTable[i] = Math.sin(ang);
        }
      }

      /**
       * In-place forward or inverse FFT on Float64Array buffers (re, im).
       * For inverse=true, conjugates twiddles and divides by N.
       */
      transform(re, im, inverse = false) {
        const n = this.size;
        const bitRev = this.bitRev;
        for (let i = 0; i < n; i++) {
          const j = bitRev[i];
          if (j > i) {
            const tr = re[i]; re[i] = re[j]; re[j] = tr;
            const ti = im[i]; im[i] = im[j]; im[j] = ti;
          }
        }
        const cosT = this.cosTable;
        const sinT = this.sinTable;
        const sign = inverse ? -1.0 : 1.0;

        for (let len = 2; len <= n; len <<= 1) {
          const halfLen = len >> 1;
          const step = n / len;
          for (let i = 0; i < n; i += len) {
            let tw = 0;
            for (let j = 0; j < halfLen; j++) {
              const wr = cosT[tw];
              const wi = sign * sinT[tw];
              const idx1 = i + j;
              const idx2 = idx1 + halfLen;
              const xr = re[idx2];
              const xi = im[idx2];
              const tr = xr * wr - xi * wi;
              const ti = xr * wi + xi * wr;
              re[idx2] = re[idx1] - tr;
              im[idx2] = im[idx1] - ti;
              re[idx1] += tr;
              im[idx1] += ti;
              tw += step;
            }
          }
        }

        if (inverse) {
          const invN = 1.0 / n;
          for (let i = 0; i < n; i++) {
            re[i] *= invN;
            im[i] *= invN;
          }
        }
      }
    }

    // =========================================================================
    // 2. 4TH-ORDER BUTTERWORTH LOWPASS COEFFICIENT DESIGN (2 Cascaded Biquads)
    // =========================================================================
    function designButterworth4LP(fsNative, fc) {
      const clampedFc = Math.min(Math.max(fc, 20), 0.48 * fsNative);
      const w0 = 2.0 * Math.PI * clampedFc / fsNative;
      const cosW0 = Math.cos(w0);
      const sinW0 = Math.sin(w0);
      // Exact 4th-order Butterworth pole Q factors: 1/(2 cos(pi/8)), 1/(2 cos(3pi/8))
      const qs = [0.541196100146197, 1.3065629648763766];
      return qs.map(Q => {
        const alpha = sinW0 / (2.0 * Q);
        const a0 = 1.0 + alpha;
        const b0 = (1.0 - cosW0) * 0.5 / a0;
        const b1 = (1.0 - cosW0) / a0;
        const b2 = b0;
        const a1 = (-2.0 * cosW0) / a0;
        const a2 = (1.0 - alpha) / a0;
        return { b0, b1, b2, a1, a2 };
      });
    }

    // =========================================================================
    // 3. BIT-EXACT D=3 ORDINAL REVERSAL LUT & FAST SYMMETRIC PRNG
    // =========================================================================
    function revOrd3Code(c) {
      const b2 = (c >> 2) & 1;
      const b1 = (c >> 1) & 1;
      const b0 = c & 1;
      return ((1 - b1) << 2) | ((1 - b2) << 1) | (1 - b0);
    }

    const REV_LUT = new Uint8Array(64);
    for (let p = 0; p < 64; p++) {
      const c1 = (p >> 3) & 7;
      const c2 = p & 7;
      REV_LUT[p] = (revOrd3Code(c2) << 3) | revOrd3Code(c1);
    }

    // =========================================================================
    // 4. ROLLING EMA SPECTRAL (3RD + 4TH) & CAUSAL TIME-ARROW ENGINE
    // =========================================================================
    class RollingSpectralEngine {
      constructor(fsA = 8000, tauSec = 0.6, gateDb = -55.0) {
        this.nFft = 256;
        this.hop = 128;
        this.alpha = 0.05;
        this.biMaxSum = 64;   // 1 <= i2 <= i1, i1 + i2 <= 64 -> 1024 triads
        this.triMaxSum = 36;  // 1 <= i3 <= i2 <= i1, i1 + i2 + i3 <= 36

        this.fft = new Radix2FFT(this.nFft);
        this.fftRe = new Float64Array(this.nFft);
        this.fftIm = new Float64Array(this.nFft);

        // Periodic Hann window (exact zero-endpoint avoidance)
        this.win = new Float64Array(this.nFft);
        let wEnergy = 0.0;
        for (let n = 0; n < this.nFft; n++) {
          const w = 0.5 * (1.0 - Math.cos(2.0 * Math.PI * n / this.nFft));
          this.win[n] = w;
          wEnergy += w * w;
        }
        this.wEnergy = wEnergy;

        // Precompute window autocorrelation rho(m) at m * hop
        const maxLag = Math.floor((this.nFft - 1) / this.hop);
        this.winRho = new Float64Array(maxLag + 1);
        this.winRho[0] = 1.0;
        for (let m = 1; m <= maxLag; m++) {
          const shift = m * this.hop;
          let dot = 0.0;
          for (let n = 0; n < this.nFft - shift; n++) {
            dot += this.win[n] * this.win[n + shift];
          }
          this.winRho[m] = dot / wEnergy;
        }

        // Precompute flat 3rd-order principal-triangle indices
        this.biW = 64; // texel x = i1 - 1 (i1 = 1..63 -> 64 cols)
        this.biH = 32; // texel y = i2 - 1 (i2 = 1..32 -> 32 rows)
        const biI1 = [], biI2 = [], biI3 = [];
        this.biValidGrid = new Uint8Array(this.biW * this.biH);
        this.biCellToFlat = new Int32Array(this.biW * this.biH).fill(-1);
        for (let i2 = 1; i2 <= Math.floor(this.biMaxSum / 2); i2++) {
          for (let i1 = i2; i1 <= this.biMaxSum - i2; i1++) {
            const flatIdx = biI1.length;
            biI1.push(i1);
            biI2.push(i2);
            biI3.push(i1 + i2);
            const cell = (i2 - 1) * this.biW + (i1 - 1);
            this.biValidGrid[cell] = 1;
            this.biCellToFlat[cell] = flatIdx;
          }
        }
        this.biI1 = new Uint16Array(biI1);
        this.biI2 = new Uint16Array(biI2);
        this.biI3 = new Uint16Array(biI3);
        this.nTriads = biI1.length;

        // Precompute flat 4th-order positive sum-octant indices
        this.triW = 36; // texel x = i1 - 1 (i1 = 1..34)
        this.triH = 18; // texel y = i2 - 1 (i2 = 1..17)
        const triI1 = [], triI2 = [], triI3 = [], triI4 = [];
        this.triValidProj = new Uint8Array(this.triW * this.triH);
        for (let i3 = 1; i3 <= Math.floor(this.triMaxSum / 3); i3++) {
          const i2Max = Math.floor((this.triMaxSum - i3) / 2);
          for (let i2 = i3; i2 <= i2Max; i2++) {
            const i1Max = this.triMaxSum - i3 - i2;
            for (let i1 = i2; i1 <= i1Max; i1++) {
              triI1.push(i1);
              triI2.push(i2);
              triI3.push(i3);
              triI4.push(i1 + i2 + i3);
              this.triValidProj[(i2 - 1) * this.triW + (i1 - 1)] = 1;
            }
          }
        }
        this.triI1 = new Uint16Array(triI1);
        this.triI2 = new Uint16Array(triI2);
        this.triI3 = new Uint16Array(triI3);
        this.triI4 = new Uint16Array(triI4);
        this.nQuartets = triI1.length;

        // Preallocated EMA state buffers
        this.NbiRe = new Float64Array(this.nTriads);
        this.NbiIm = new Float64Array(this.nTriads);
        this.DUbi  = new Float64Array(this.nTriads);
        this.DVbi  = new Float64Array(this.nTriads);
        this.b2Arr = new Float32Array(this.nTriads);
        this.biphaseArr = new Float32Array(this.nTriads);

        this.NtriRe = new Float64Array(this.nQuartets);
        this.NtriIm = new Float64Array(this.nQuartets);
        this.DUtri  = new Float64Array(this.nQuartets);
        this.DVtri  = new Float64Array(this.nQuartets);
        this.t2Arr  = new Float32Array(this.nQuartets);
        this.triphaseArr = new Float32Array(this.nQuartets);

        // 2D max-projection buffers for 4th-order WebGL texture & hover inspection
        this.triProjVal   = new Float32Array(this.triW * this.triH);
        this.triProjPhase = new Float32Array(this.triW * this.triH);
        this.triProjBestI3 = new Uint16Array(this.triW * this.triH);
        this.triProjSig   = new Uint8Array(this.triW * this.triH);

        // RGBA8 packed textures for WebGL 2.0 (universal mobile GPU compatibility)
        this.biTexRGBA  = new Uint8Array(this.biW * this.biH * 4);
        this.triTexRGBA = new Uint8Array(this.triW * this.triH * 4);

        // 2nd-order rolling RTA Power Spectrum (129 bins) & Scope Ring Buffer
        this.rtaBins = (this.nFft >> 1) + 1;
        this.rtaPower = new Float64Array(this.rtaBins);
        this.rtaRefDb = null;

        this.scopeLen = 1024; // ~128 ms @ 8 kHz
        this.scopeRing = new Float32Array(this.scopeLen);
        this.scopeWrite = 0;

        // Crest factor sliding peak window (~0.64s max-hold over 80 blocks of 64 samples)
        this.peakRingLen = 80;
        this.peakRing = new Float32Array(this.peakRingLen);
        this.peakWrite = 0;

        // STFT hop ring buffer
        this.stftRing = new Float64Array(this.nFft);
        this.hopFill = 0;
        this.totalSamples = 0;

        // Causal zero-backtracking Time-Arrow state
        this.ordTail = new Float64Array(3);
        this.pOrd = new Float64Array(64);
        this.rngState = 0x13579bdf;

        this.configure(fsA, tauSec, gateDb);
        this.reset();
      }

      configure(fsA, tauSec, gateDb) {
        this.fsA = Math.max(500, fsA);
        this.df = this.fsA / this.nFft;
        this.tauSec = Math.min(2.5, Math.max(0.2, tauSec));
        this.gateDb = gateDb;
        this.gateRms = Math.pow(10.0, gateDb / 20.0);

        const dtHop = this.hop / this.fsA;
        this.lamHop = 1.0 - Math.exp(-dtHop / this.tauSec);

        // Exact EMA-weighted overlap autocorrelation denominators for p=3 and p=4
        let c3 = 0.0, c4 = 0.0;
        for (let m = 1; m < this.winRho.length; m++) {
          const rho = this.winRho[m];
          const decM = Math.pow(1.0 - this.lamHop, m);
          c3 += decM * rho * rho * rho;
          c4 += decM * rho * rho * rho * rho;
        }
        this.ovlpFactor3 = 1.0 + 2.0 * c3;
        this.ovlpFactor4 = 1.0 + 2.0 * c4;
      }

      reset() {
        this.NbiRe.fill(0); this.NbiIm.fill(0); this.DUbi.fill(0); this.DVbi.fill(0);
        this.b2Arr.fill(0); this.biphaseArr.fill(0);
        this.NtriRe.fill(0); this.NtriIm.fill(0); this.DUtri.fill(0); this.DVtri.fill(0);
        this.t2Arr.fill(0); this.triphaseArr.fill(0);
        this.triProjVal.fill(0); this.triProjPhase.fill(0);
        this.triProjBestI3.fill(0); this.triProjSig.fill(0);
        this.rtaPower.fill(1e-12);
        this.scopeRing.fill(0);
        this.scopeWrite = 0;
        this.peakRing.fill(0);
        this.peakWrite = 0;
        this.stftRing.fill(0);
        this.hopFill = 0;
        this.totalSamples = 0;
        this.activeHops = 0;
        this.isGated = false;

        // Time-arrow reset
        this.skewTail = 0.0;
        this.skewPrimed = false;
        this.m2Dx = 0.0;
        this.m3Dx = 0.0;
        this.msSignal = 0.0;
        this.ordTail.fill(0);
        this.ordPrimed = false;
        this.pOrd.fill(1.0 / 64.0);
        this.hSpecEma = 0.0;
        this.lastHopMs = 0.0;

        this.latestReadout = this._emptyReadout();
        this._packTextures(1.0, 1.0);
      }

      _emptyReadout() {
        return {
          kEff3: 0, kEff4: 0,
          critBi: 1, critTri: 1,
          biHits: 0, triHits: 0,
          b2Max: 0, t2Max: 0,
          topTriad: null, topQuartet: null,
          phiRev: 0, dJs: 0, imRms: 0, imPolarity: 0,
          crest: 0, hSpec: 0,
          verdict: "IDLE / STANDBY", verdictClass: "v-clean",
          gated: false, hopMs: 0
        };
      }

      /** Fast symmetric zero-mean triangular dither in [-1, +1] (Xorshift32). */
      _symDither() {
        let s = this.rngState;
        s ^= s << 13; s ^= s >>> 17; s ^= s << 5;
        const u1 = (s >>> 0) * 2.3283064365386963e-10;
        s ^= s << 13; s ^= s >>> 17; s ^= s << 5;
        const u2 = (s >>> 0) * 2.3283064365386963e-10;
        this.rngState = s;
        return u1 - u2;
      }

      _kEff(order) {
        const n = this.activeHops;
        if (n <= 1) return n;
        const lam = this.lamHop;
        const r = 1.0 - lam;
        const rn = Math.pow(r, n);
        const num = (1.0 - rn) * (1.0 - rn);
        const den = Math.max(1.0 - rn * rn, 1e-30);
        const kEma = ((2.0 - lam) / lam) * (num / den);
        const ovlp = order === 3 ? this.ovlpFactor3 : this.ovlpFactor4;
        return Math.max(1.0, kEma / ovlp);
      }

      /** Causal O(B) update for phi_rev, D=3 Bandt-Pompe ordinal histogram, and Crest Factor. */
      _updateTimeArrowChunk(chunk) {
        const B = chunk.length;
        if (B === 0) return;

        // Always feed oscilloscope ring buffer so live waveform stays responsive
        const sLen = this.scopeLen;
        let sw = this.scopeWrite;
        let sumSq = 0.0;
        let blkPeak = 0.0;
        for (let i = 0; i < B; i++) {
          const v = chunk[i];
          this.scopeRing[sw] = v;
          sw = (sw + 1) & (sLen - 1);
          sumSq += v * v;
          const av = v < 0 ? -v : v;
          if (av > blkPeak) blkPeak = av;
        }
        this.scopeWrite = sw;

        const rms = Math.sqrt(sumSq / B);
        if (rms < this.gateRms || B < 4) {
          this.isGated = true;
          return;
        }
        this.isGated = false;

        // Sliding peak window for Crest Factor
        this.peakRing[this.peakWrite] = blkPeak;
        this.peakWrite = (this.peakWrite + 1) % this.peakRingLen;

        const lamB = 1.0 - Math.exp(-(B / this.fsA) / this.tauSec);
        const decB = 1.0 - lamB;
        this.msSignal = decB * this.msSignal + lamB * (sumSq / B);

        // 1. 1-lag increment skewness (1-sample carry tail across block boundary)
        let s2 = 0.0, s3 = 0.0, cntDx = 0;
        let prev = this.skewTail;
        let startIdx = 0;
        if (!this.skewPrimed) {
          prev = chunk[0];
          startIdx = 1;
          this.skewPrimed = true;
        }
        for (let i = startIdx; i < B; i++) {
          const cur = chunk[i];
          const dx = cur - prev;
          const dx2 = dx * dx;
          s2 += dx2;
          s3 += dx2 * dx;
          cntDx++;
          prev = cur;
        }
        this.skewTail = chunk[B - 1];
        if (cntDx > 0) {
          this.m2Dx = decB * this.m2Dx + lamB * (s2 / cntDx);
          this.m3Dx = decB * this.m3Dx + lamB * (s3 / cntDx);
        }

        // 2. D=3 Bandt-Pompe ordinal transition histogram (3-sample carry + sub-LSB dither)
        const ditherAmp = 1e-6 * rms;
        const hist = new Float64Array(64);
        let nPairs = 0;
        // Build tiny local view of [3 carry samples + B dithered samples]
        const nExt = this.ordPrimed ? (B + 3) : B;
        const buf = new Float64Array(nExt);
        let off = 0;
        if (this.ordPrimed) {
          buf[0] = this.ordTail[0];
          buf[1] = this.ordTail[1];
          buf[2] = this.ordTail[2];
          off = 3;
        } else {
          this.ordPrimed = true;
        }
        for (let i = 0; i < B; i++) {
          buf[off + i] = chunk[i] + ditherAmp * this._symDither();
        }
        // Save last 3 dithered samples as exact seam carry for next block
        this.ordTail[0] = buf[nExt - 3];
        this.ordTail[1] = buf[nExt - 2];
        this.ordTail[2] = buf[nExt - 1];

        let prevCode = -1;
        for (let i = 0; i <= nExt - 3; i++) {
          const x0 = buf[i], x1 = buf[i + 1], x2 = buf[i + 2];
          const code = ((x0 < x1 ? 1 : 0) << 2) | ((x1 < x2 ? 1 : 0) << 1) | (x0 < x2 ? 1 : 0);
          if (prevCode >= 0) {
            hist[(prevCode << 3) | code] += 1.0;
            nPairs++;
          }
          prevCode = code;
        }
        if (nPairs > 0) {
          const invPairs = 1.0 / nPairs;
          for (let k = 0; k < 64; k++) {
            this.pOrd[k] = decB * this.pOrd[k] + lamB * (hist[k] * invPairs);
          }
        }
      }

      /** Triggered every exact hop (128 decimated samples). */
      _processHop() {
        const t0 = performance.now();
        const N = this.nFft;
        const ring = this.stftRing;

        // Per-frame DC removal & silence gate check
        let mean = 0.0;
        for (let i = 0; i < N; i++) mean += ring[i];
        mean /= N;

        let sumSq = 0.0;
        const re = this.fftRe;
        const im = this.fftIm;
        const win = this.win;
        for (let i = 0; i < N; i++) {
          const xc = ring[i] - mean;
          sumSq += xc * xc;
          re[i] = xc * win[i];
          im[i] = 0.0;
        }
        const rms = Math.sqrt(sumSq / N);
        if (rms < this.gateRms) {
          this.isGated = true;
          this.lastHopMs = performance.now() - t0;
          return;
        }
        this.isGated = false;

        // Exact in-place radix-2 FFT
        this.fft.transform(re, im, false);

        const lam = this.lamHop;
        const dec = 1.0 - lam;
        this.activeHops++;

        // 2nd-order rolling RTA power + normalized spectral entropy H_spec
        let pSum = 0.0;
        const rtaP = this.rtaPower;
        const invScale = 1.0 / (N * N);
        for (let b = 0; b < this.rtaBins; b++) {
          const p = (re[b] * re[b] + im[b] * im[b]) * invScale;
          rtaP[b] = dec * rtaP[b] + lam * p;
          if (b >= 1 && b < this.rtaBins - 1) pSum += p;
        }
        if (pSum > 1e-30) {
          let ent = 0.0;
          const invP = 1.0 / pSum;
          for (let b = 1; b < this.rtaBins - 1; b++) {
            const pb = (re[b] * re[b] + im[b] * im[b]) * invScale * invP;
            if (pb > 1e-15) ent -= pb * Math.log(pb);
          }
          const hNorm = ent / Math.log(this.rtaBins - 2);
          this.hSpecEma = dec * this.hSpecEma + lam * hNorm;
        }

        // 3rd-order flat vectorized EMA update
        const biI1 = this.biI1, biI2 = this.biI2, biI3 = this.biI3;
        const NbiRe = this.NbiRe, NbiIm = this.NbiIm, DUbi = this.DUbi, DVbi = this.DVbi;
        for (let k = 0, len = this.nTriads; k < len; k++) {
          const a = biI1[k], b = biI2[k], c = biI3[k];
          const xaR = re[a], xaI = im[a];
          const xbR = re[b], xbI = im[b];
          const uR = xaR * xbR - xaI * xbI;
          const uI = xaR * xbI + xaI * xbR;
          const vR = re[c], vI = im[c];
          // U3 * conj(V3)
          const pR = uR * vR + uI * vI;
          const pI = uI * vR - uR * vI;
          NbiRe[k] = dec * NbiRe[k] + lam * pR;
          NbiIm[k] = dec * NbiIm[k] + lam * pI;
          DUbi[k]  = dec * DUbi[k]  + lam * (uR * uR + uI * uI);
          DVbi[k]  = dec * DVbi[k]  + lam * (vR * vR + vI * vI);
        }

        // 4th-order flat vectorized EMA update
        const triI1 = this.triI1, triI2 = this.triI2, triI3 = this.triI3, triI4 = this.triI4;
        const NtriRe = this.NtriRe, NtriIm = this.NtriIm, DUtri = this.DUtri, DVtri = this.DVtri;
        for (let k = 0, len = this.nQuartets; k < len; k++) {
          const a = triI1[k], b = triI2[k], c = triI3[k], d = triI4[k];
          const xaR = re[a], xaI = im[a];
          const xbR = re[b], xbI = im[b];
          const abR = xaR * xbR - xaI * xbI;
          const abI = xaR * xbI + xaI * xbR;
          const xcR = re[c], xcI = im[c];
          const uR = abR * xcR - abI * xcI;
          const uI = abR * xcI + abI * xcR;
          const vR = re[d], vI = im[d];
          // U4 * conj(V4)
          const pR = uR * vR + uI * vI;
          const pI = uI * vR - uR * vI;
          NtriRe[k] = dec * NtriRe[k] + lam * pR;
          NtriIm[k] = dec * NtriIm[k] + lam * pI;
          DUtri[k]  = dec * DUtri[k]  + lam * (uR * uR + uI * uI);
          DVtri[k]  = dec * DVtri[k]  + lam * (vR * vR + vI * vI);
        }

        this.lastHopMs = performance.now() - t0;
      }

      /** Push decimated audio chunk (Float32Array) into the engine. */
      pushDecimated(chunk) {
        this._updateTimeArrowChunk(chunk);
        const n = chunk.length;
        let pos = 0;
        const N = this.nFft;
        const ring = this.stftRing;
        while (pos < n) {
          const take = Math.min(this.hop - this.hopFill, n - pos);
          if (take >= N) {
            for (let i = 0; i < N; i++) ring[i] = chunk[pos + take - N + i];
          } else {
            ring.copyWithin(0, take);
            for (let i = 0; i < take; i++) ring[N - take + i] = chunk[pos + i];
          }
          this.hopFill += take;
          this.totalSamples += take;
          pos += take;
          if (this.hopFill === this.hop) {
            this.hopFill = 0;
            if (this.totalSamples >= N) {
              this._processHop();
            }
          }
        }
      }

      /** Recompute derived statistics, pack WebGL RGBA8 textures, return HUD readout. */
      computeReadout() {
        const k3 = this._kEff(3);
        const k4 = this._kEff(4);
        const critBi  = k3 > 1.05 ? (1.0 - Math.pow(this.alpha / this.nTriads,   1.0 / (k3 - 1.0))) : 1.0;
        const critTri = k4 > 1.05 ? (1.0 - Math.pow(this.alpha / this.nQuartets, 1.0 / (k4 - 1.0))) : 1.0;

        // 1. 3rd-Order Bicoherence summary
        let b2Max = 0.0, topBiIdx = 0, biHits = 0;
        let imSqSum = 0.0, imSignedNum = 0.0, b2Sum = 0.0;
        let sigSinSum = 0.0, sigSinCnt = 0;

        for (let k = 0; k < this.nTriads; k++) {
          const nR = this.NbiRe[k], nI = this.NbiIm[k];
          const den = this.DUbi[k] * this.DVbi[k];
          let b2 = den > 1e-30 ? (nR * nR + nI * nI) / den : 0.0;
          if (b2 > 1.0) b2 = 1.0;
          const psi = Math.atan2(nI, nR);
          this.b2Arr[k] = b2;
          this.biphaseArr[k] = psi;

          const sPsi = Math.sin(psi);
          imSqSum += b2 * sPsi * sPsi;
          imSignedNum += b2 * sPsi;
          b2Sum += b2;

          if (b2 >= critBi) {
            biHits++;
            sigSinSum += Math.abs(sPsi);
            sigSinCnt++;
          }
          if (b2 > b2Max) {
            b2Max = b2;
            topBiIdx = k;
          }
        }
        const imRms = Math.sqrt(imSqSum / this.nTriads);
        const imPolarity = b2Sum > 1e-12 ? (imSignedNum / b2Sum) : 0.0;
        const sigImMean = sigSinCnt > 0 ? (sigSinSum / sigSinCnt) : 0.0;

        // 2. 4th-Order Tricoherence summary + 2D max projection onto (i1, i2)
        this.triProjVal.fill(0);
        this.triProjPhase.fill(0);
        this.triProjBestI3.fill(0);
        this.triProjSig.fill(0);

        let t2Max = 0.0, topTriIdx = 0, triHits = 0;
        for (let k = 0; k < this.nQuartets; k++) {
          const nR = this.NtriRe[k], nI = this.NtriIm[k];
          const den = this.DUtri[k] * this.DVtri[k];
          let t2 = den > 1e-30 ? (nR * nR + nI * nI) / den : 0.0;
          if (t2 > 1.0) t2 = 1.0;
          const psi = Math.atan2(nI, nR);
          this.t2Arr[k] = t2;
          this.triphaseArr[k] = psi;

          const isSig = t2 >= critTri ? 1 : 0;
          if (isSig) triHits++;
          if (t2 > t2Max) {
            t2Max = t2;
            topTriIdx = k;
          }

          const cell = (this.triI2[k] - 1) * this.triW + (this.triI1[k] - 1);
          if (t2 >= this.triProjVal[cell]) {
            this.triProjVal[cell] = t2;
            this.triProjPhase[cell] = psi;
            this.triProjBestI3[cell] = this.triI3[k];
          }
          if (isSig) this.triProjSig[cell] = 1;
        }

        // 3. Causal Time-Arrow metrics
        const phiRev = this.m2Dx > 1e-24 ? (this.m3Dx / Math.pow(this.m2Dx, 1.5)) : 0.0;

        let pTot = 0.0;
        for (let k = 0; k < 64; k++) pTot += this.pOrd[k];
        let dJs = 0.0;
        if (pTot > 1e-30) {
          const invP = 1.0 / pTot;
          let klF = 0.0, klB = 0.0;
          for (let k = 0; k < 64; k++) {
            const pf = this.pOrd[k] * invP;
            const pb = this.pOrd[REV_LUT[k]] * invP;
            const m = 0.5 * (pf + pb);
            if (pf > 1e-15 && m > 1e-15) klF += pf * Math.log(pf / m);
            if (pb > 1e-15 && m > 1e-15) klB += pb * Math.log(pb / m);
          }
          dJs = Math.max(0.0, 0.5 * (klF + klB));
        }

        // Rolling Crest Factor
        let maxPeak = 0.0;
        for (let i = 0; i < this.peakRingLen; i++) {
          if (this.peakRing[i] > maxPeak) maxPeak = this.peakRing[i];
        }
        const rmsEma = Math.sqrt(Math.max(this.msSignal, 1e-30));
        const crest = rmsEma > 1e-9 ? Math.min(30.0, maxPeak / rmsEma) : 0.0;

        // 4. Physics-based Diagnostic Verdict Classifier
        let verdict = "CLEAN / UNCOUPLED";
        let verdictClass = "v-clean";

        if (this.activeHops < 6) {
          verdict = this.activeHops === 0 ? "IDLE / STANDBY" : "WARMING UP...";
          verdictClass = "v-clean";
        } else if (phiRev <= -0.24 && (dJs >= 0.0025 || imRms >= 0.05)) {
          verdict = "REVERSED ARROW";
          verdictClass = "v-reverse";
        } else if (phiRev >= 0.24 && (dJs >= 0.0025 || imRms >= 0.05)) {
          verdict = "CAUSAL PUNCH";
          verdictClass = "v-causal";
        } else if (biHits >= 3 && b2Max >= 0.28) {
          // Separate static asymmetric x^2 distortion (real bispectrum, psi ~ 0/180 deg)
          // from phase-dispersed / pre-ringing transients (scattered or quadrature biphases)
          if (sigImMean < 0.34) {
            verdict = "EVEN / IMD (3rd)";
            verdictClass = "v-even";
          } else {
            verdict = "PHASE SMEARED";
            verdictClass = "v-smeared";
          }
        } else if (triHits >= 3 && t2Max >= 0.28) {
          verdict = "ODD CLIP (4th)";
          verdictClass = "v-odd";
        } else if (crest >= 2.15 && crest <= 3.9 && Math.abs(phiRev) < 0.24 && imRms >= 0.06) {
          verdict = "PHASE SMEARED";
          verdictClass = "v-smeared";
        }

        // Pack RGBA8 WebGL textures
        this._packTextures(critBi, critTri);

        const df = this.df;
        this.latestReadout = {
          kEff3: k3, kEff4: k4,
          critBi, critTri,
          biHits, triHits,
          b2Max, t2Max,
          topTriad: this.activeHops > 0 ? {
            f1: this.biI1[topBiIdx] * df,
            f2: this.biI2[topBiIdx] * df,
            f3: this.biI3[topBiIdx] * df,
            b2: b2Max,
            psiDeg: this.biphaseArr[topBiIdx] * (180 / Math.PI)
          } : null,
          topQuartet: this.activeHops > 0 ? {
            f1: this.triI1[topTriIdx] * df,
            f2: this.triI2[topTriIdx] * df,
            f3: this.triI3[topTriIdx] * df,
            f4: this.triI4[topTriIdx] * df,
            t2: t2Max,
            psiDeg: this.triphaseArr[topTriIdx] * (180 / Math.PI)
          } : null,
          phiRev, dJs, imRms, imPolarity,
          crest, hSpec: this.hSpecEma,
          verdict, verdictClass,
          gated: this.isGated,
          hopMs: this.lastHopMs
        };
        return this.latestReadout;
      }

      /** Pack coherence, Bonferroni significance, |sin(phase)|, and valid mask into RGBA8. */
      _packTextures(critBi, critTri) {
        // 3rd-order Bicoherence RGBA8
        const biTex = this.biTexRGBA;
        biTex.fill(0);
        for (let k = 0; k < this.nTriads; k++) {
          const i1 = this.biI1[k], i2 = this.biI2[k];
          const off = ((i2 - 1) * this.biW + (i1 - 1)) * 4;
          const b2 = this.b2Arr[k];
          const asym = Math.abs(Math.sin(this.biphaseArr[k]));
          biTex[off]     = Math.min(255, Math.max(0, Math.round(b2 * 255)));
          biTex[off + 1] = b2 >= critBi ? 255 : 0;
          biTex[off + 2] = Math.min(255, Math.max(0, Math.round(asym * 255)));
          biTex[off + 3] = 255;
        }

        // 4th-order Tricoherence 2D Max-Projection RGBA8
        const triTex = this.triTexRGBA;
        triTex.fill(0);
        const totalCells = this.triW * this.triH;
        for (let cell = 0; cell < totalCells; cell++) {
          if (!this.triValidProj[cell]) continue;
          const off = cell * 4;
          const t2 = this.triProjVal[cell];
          const asym = Math.abs(Math.sin(this.triProjPhase[cell]));
          triTex[off]     = Math.min(255, Math.max(0, Math.round(t2 * 255)));
          triTex[off + 1] = this.triProjSig[cell] ? 255 : 0;
          triTex[off + 2] = Math.min(255, Math.max(0, Math.round(asym * 255)));
          triTex[off + 3] = 255;
        }
      }

      inspectBi(cellX, cellY) {
        if (cellX < 0 || cellX >= this.biW || cellY < 0 || cellY >= this.biH) return null;
        const flat = this.biCellToFlat[cellY * this.biW + cellX];
        if (flat < 0) return null;
        const i1 = this.biI1[flat], i2 = this.biI2[flat], i3 = this.biI3[flat];
        const b2 = this.b2Arr[flat];
        const psi = this.biphaseArr[flat];
        return {
          f1: i1 * this.df, f2: i2 * this.df, f3: i3 * this.df,
          val: b2, psiDeg: psi * (180 / Math.PI),
          asym: Math.abs(Math.sin(psi)),
          sig: b2 >= this.latestReadout.critBi
        };
      }

      inspectTri(cellX, cellY) {
        if (cellX < 0 || cellX >= this.triW || cellY < 0 || cellY >= this.triH) return null;
        const cell = cellY * this.triW + cellX;
        if (!this.triValidProj[cell]) return null;
        const i1 = cellX + 1, i2 = cellY + 1;
        const i3 = this.triProjBestI3[cell] || 1;
        const t2 = this.triProjVal[cell];
        const psi = this.triProjPhase[cell];
        return {
          f1: i1 * this.df, f2: i2 * this.df, f3: i3 * this.df, f4: (i1 + i2 + i3) * this.df,
          val: t2, psiDeg: psi * (180 / Math.PI),
          asym: Math.abs(Math.sin(psi)),
          sig: Boolean(this.triProjSig[cell])
        };
      }
    }

    // =========================================================================
    // 5. WEBGL 2.0 (#version 300 es) GPU HEATMAP RENDERER (+ 2D Canvas Fallback)
    // =========================================================================
    const VS_300 = `#version 300 es
      precision highp float;
      const vec2 POS[4] = vec2[4](
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2(-1.0,  1.0),
        vec2( 1.0,  1.0)
      );
      out vec2 v_uv;
      void main() {
        vec2 p = POS[gl_VertexID];
        v_uv = p * 0.5 + 0.5;
        gl_Position = vec4(p, 0.0, 1.0);
      }
    `;

    const FS_300 = `#version 300 es
      precision highp float;
      uniform sampler2D u_tex;
      uniform vec2 u_gridSize;
      uniform float u_asymMode;
      uniform vec2 u_hoverCell;
      in vec2 v_uv;
      out vec4 outColor;

      // Dark scientific studio colormap: obsidian -> indigo -> cyan -> amber -> crimson -> white
      vec3 scientificMap(float x) {
        float t = clamp(x, 0.0, 1.0);
        vec3 c0 = vec3(0.031, 0.047, 0.086); // obsidian
        vec3 c1 = vec3(0.118, 0.145, 0.420); // deep indigo
        vec3 c2 = vec3(0.047, 0.651, 0.792); // electric cyan
        vec3 c3 = vec3(0.961, 0.620, 0.043); // amber
        vec3 c4 = vec3(0.937, 0.267, 0.267); // hot crimson
        vec3 c5 = vec3(1.000, 0.968, 0.925); // white-hot
        if (t < 0.24) return mix(c0, c1, t / 0.24);
        if (t < 0.50) return mix(c1, c2, (t - 0.24) / 0.26);
        if (t < 0.74) return mix(c2, c3, (t - 0.50) / 0.24);
        if (t < 0.90) return mix(c3, c4, (t - 0.74) / 0.16);
        return mix(c4, c5, (t - 0.90) / 0.10);
      }

      float sampleMask(vec2 cell) {
        if (cell.x < 0.0 || cell.y < 0.0 || cell.x >= u_gridSize.x || cell.y >= u_gridSize.y) {
          return 0.0;
        }
        vec2 uv = (cell + 0.5) / u_gridSize;
        return texture(u_tex, uv).a;
      }

      void main() {
        vec2 fCell = v_uv * u_gridSize;
        vec2 cell = floor(fCell);
        vec2 frac = fract(fCell);
        vec2 texUv = (cell + 0.5) / u_gridSize;

        vec4 data = texture(u_tex, texUv);
        float coh = data.r;
        float sig = data.g;
        float asym = data.b;
        float valid = data.a;

        // Invalid region outside principal triangle: sleek dark hatched drafting field
        if (valid < 0.5) {
          float hatch = step(0.92, fract((v_uv.x * 40.0) - (v_uv.y * 25.0)));
          vec3 bg = vec3(0.024, 0.035, 0.058) + vec3(0.014, 0.022, 0.038) * hatch;
          // Draw crisp outer boundary adjacent to valid principal-triangle cells
          float nL = sampleMask(cell + vec2(-1.0,  0.0));
          float nR = sampleMask(cell + vec2( 1.0,  0.0));
          float nD = sampleMask(cell + vec2( 0.0, -1.0));
          float nU = sampleMask(cell + vec2( 0.0,  1.0));
          float border = 0.0;
          if (nL > 0.5 && frac.x < 0.16) border = 1.0;
          if (nR > 0.5 && frac.x > 0.84) border = 1.0;
          if (nD > 0.5 && frac.y < 0.16) border = 1.0;
          if (nU > 0.5 && frac.y > 0.84) border = 1.0;
          bg = mix(bg, vec3(0.24, 0.44, 0.62), border * 0.85);
          outColor = vec4(bg, 1.0);
          return;
        }

        // Effective value: standard coherence or Im[B] asymmetry-weighted (coh * sin^2(phase))
        float rawVal = (u_asymMode > 0.5) ? (coh * asym * asym) : coh;
        float mapped = pow(clamp(rawVal, 0.0, 1.0), 0.76);
        vec3 col = scientificMap(mapped);

        // Subtle pixel grid inside valid triangle
        float cellEdge = max(
          step(0.92, frac.x) + step(frac.x, 0.08),
          step(0.92, frac.y) + step(frac.y, 0.08)
        );
        col *= (1.0 - 0.12 * min(cellEdge, 1.0));

        // Luminous contour highlight on Bonferroni-significant cells
        if (sig > 0.5) {
          float boxContour = max(
            step(0.84, frac.x) + step(frac.x, 0.16),
            step(0.84, frac.y) + step(frac.y, 0.16)
          );
          vec3 glowCol = (u_asymMode > 0.5) ? vec3(1.0, 0.82, 0.35) : vec3(0.98, 0.96, 1.0);
          col = mix(col, glowCol, min(boxContour, 1.0) * 0.68);
        }

        // Hover / touch crosshair and active cell outline
        if (u_hoverCell.x >= 0.0) {
          float sameX = step(abs(cell.x - u_hoverCell.x), 0.25);
          float sameY = step(abs(cell.y - u_hoverCell.y), 0.25);
          float cross = max(sameX, sameY);
          col = mix(col, vec3(0.14, 0.84, 0.96), cross * 0.22);
          if (sameX > 0.5 && sameY > 0.5) {
            float hiBox = max(
              step(0.82, frac.x) + step(frac.x, 0.18),
              step(0.82, frac.y) + step(frac.y, 0.18)
            );
            col = mix(col, vec3(1.0, 1.0, 1.0), min(hiBox, 1.0));
          }
        }

        outColor = vec4(col, 1.0);
      }
    `;

    class HeatmapRenderer {
      constructor(canvas, gridW, gridH, tagEl) {
        this.canvas = canvas;
        this.gridW = gridW;
        this.gridH = gridH;
        this.hoverX = -1;
        this.hoverY = -1;
        this.isWebGL2 = false;

        const gl = canvas.getContext("webgl2", {
          antialias: false, alpha: false, depth: false, stencil: false, preserveDrawingBuffer: false
        });

        if (gl) {
          try {
            this._initWebGL2(gl);
            this.gl = gl;
            this.isWebGL2 = true;
            if (tagEl) tagEl.textContent = "WebGL 2.0";
          } catch (err) {
            console.warn("WebGL2 shader init failed, using 2D fallback:", err);
            this._initFallback2D(tagEl);
          }
        } else {
          this._initFallback2D(tagEl);
        }
      }

      _initWebGL2(gl) {
        const compile = (type, src) => {
          const sh = gl.createShader(type);
          gl.shaderSource(sh, src);
          gl.compileShader(sh);
          if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
            throw new Error(gl.getShaderInfoLog(sh));
          }
          return sh;
        };
        const vs = compile(gl.VERTEX_SHADER, VS_300);
        const fs = compile(gl.FRAGMENT_SHADER, FS_300);
        const prog = gl.createProgram();
        gl.attachShader(prog, vs);
        gl.attachShader(prog, fs);
        gl.linkProgram(prog);
        if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
          throw new Error(gl.getProgramInfoLog(prog));
        }

        this.prog = prog;
        this.vao = gl.createVertexArray();
        this.uTex = gl.getUniformLocation(prog, "u_tex");
        this.uGrid = gl.getUniformLocation(prog, "u_gridSize");
        this.uAsym = gl.getUniformLocation(prog, "u_asymMode");
        this.uHover = gl.getUniformLocation(prog, "u_hoverCell");

        const tex = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, tex);
        gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        gl.texImage2D(
          gl.TEXTURE_2D, 0, gl.RGBA, this.gridW, this.gridH, 0,
          gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(this.gridW * this.gridH * 4)
        );
        this.tex = tex;
      }

      _initFallback2D(tagEl) {
        this.ctx2d = this.canvas.getContext("2d");
        this.isWebGL2 = false;
        if (tagEl) tagEl.textContent = "Canvas 2D";
      }

      resize() {
        const dpr = Math.min(window.devicePixelRatio || 1, 2);
        const rect = this.canvas.getBoundingClientRect();
        const w = Math.max(1, Math.round(rect.width * dpr));
        const h = Math.max(1, Math.round(rect.height * dpr));
        if (this.canvas.width !== w || this.canvas.height !== h) {
          this.canvas.width = w;
          this.canvas.height = h;
        }
      }

      render(rgbaBytes, asymMode) {
        this.resize();
        if (this.isWebGL2) {
          const gl = this.gl;
          gl.viewport(0, 0, this.canvas.width, this.canvas.height);
          gl.useProgram(this.prog);
          gl.bindVertexArray(this.vao);
          gl.activeTexture(gl.TEXTURE0);
          gl.bindTexture(gl.TEXTURE_2D, this.tex);
          gl.texSubImage2D(
            gl.TEXTURE_2D, 0, 0, 0, this.gridW, this.gridH,
            gl.RGBA, gl.UNSIGNED_BYTE, rgbaBytes
          );
          gl.uniform1i(this.uTex, 0);
          gl.uniform2f(this.uGrid, this.gridW, this.gridH);
          gl.uniform1f(this.uAsym, asymMode ? 1.0 : 0.0);
          gl.uniform2f(this.uHover, this.hoverX, this.hoverY);
          gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
        } else if (this.ctx2d) {
          this._render2D(rgbaBytes, asymMode);
        }
      }

      _render2D(rgbaBytes, asymMode) {
        const ctx = this.ctx2d;
        const W = this.canvas.width, H = this.canvas.height;
        const cw = W / this.gridW, ch = H / this.gridH;
        ctx.fillStyle = "#06090f";
        ctx.fillRect(0, 0, W, H);

        for (let y = 0; y < this.gridH; y++) {
          const py = H - (y + 1) * ch;
          for (let x = 0; x < this.gridW; x++) {
            const off = (y * this.gridW + x) * 4;
            if (rgbaBytes[off + 3] < 128) continue;
            const coh = rgbaBytes[off] / 255;
            const sig = rgbaBytes[off + 1] > 128;
            const asym = rgbaBytes[off + 2] / 255;
            const raw = asymMode ? (coh * asym * asym) : coh;
            const t = Math.pow(Math.min(1, Math.max(0, raw)), 0.76);
            ctx.fillStyle = this._mapColor2D(t);
            ctx.fillRect(x * cw, py, Math.ceil(cw), Math.ceil(ch));
            if (sig) {
              ctx.strokeStyle = asymMode ? "rgba(255,210,90,0.8)" : "rgba(255,255,255,0.75)";
              ctx.lineWidth = 1;
              ctx.strokeRect(x * cw + 0.5, py + 0.5, Math.max(1, cw - 1), Math.max(1, ch - 1));
            }
          }
        }
        if (this.hoverX >= 0 && this.hoverY >= 0) {
          const px = this.hoverX * cw;
          const py = H - (this.hoverY + 1) * ch;
          ctx.strokeStyle = "#22d3ee";
          ctx.lineWidth = 1.5;
          ctx.strokeRect(px, py, cw, ch);
        }
      }

      _mapColor2D(t) {
        const stops = [
          [0.00, [8, 12, 22]],
          [0.24, [30, 37, 107]],
          [0.50, [12, 166, 202]],
          [0.74, [245, 158, 11]],
          [0.90, [239, 68, 68]],
          [1.00, [255, 247, 236]]
        ];
        for (let i = 0; i < stops.length - 1; i++) {
          const [t0, c0] = stops[i], [t1, c1] = stops[i + 1];
          if (t <= t1) {
            const f = (t - t0) / (t1 - t0);
            const r = Math.round(c0[0] + f * (c1[0] - c0[0]));
            const g = Math.round(c0[1] + f * (c1[1] - c0[1]));
            const b = Math.round(c0[2] + f * (c1[2] - c0[2]));
            return `rgb(${r},${g},${b})`;
          }
        }
        return "rgb(255,247,236)";
      }
    }

    // =========================================================================
    // 6. SYNTH TESTBED BUFFER GENERATOR (7 Exact DSP Scenarios)
    // =========================================================================
    function createMulberry32(seed) {
      let a = seed >>> 0;
      return () => {
        a = (a + 0x6D2B79F5) >>> 0;
        let t = Math.imul(a ^ (a >>> 15), 1 | a);
        t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
      };
    }

    function gaussNoise(rng) {
      const u1 = Math.max(1e-12, rng());
      const u2 = rng();
      return Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2.0 * Math.PI * u2);
    }

    /**
     * Synthesize one exact dry 125 BPM (0.48s) causal multi-harmonic sawtooth kick:
     * onset at t = 0.08s so pre-ringing before onset is clearly visible on the scope.
     */
    function buildDryKickBeat(fs, beatSec = 0.48) {
      const N = Math.round(fs * beatSec);
      const out = new Float64Array(N);
      const tOnset = 0.08;
      // 62.5 Hz sub punch + 250 Hz multi-harmonic sawtooth click (h=1..6 -> 250..1500 Hz)
      // On 420 Hz zero-phase lowpass, harmonics >= 500 Hz vanish, cleanly collapsing the
      // time-arrow while generating visible symmetric pre-ringing ahead of 0.08s.
      for (let n = 0; n < N; n++) {
        const t = n / fs;
        if (t < tOnset) continue;
        const dt = t - tOnset;
        const attack = 1.0 - Math.exp(-dt / 0.0012);
        const envSub = attack * Math.exp(-dt / 0.085);
        const envSaw = attack * Math.exp(-dt / 0.055);

        const sub = 0.55 * Math.sin(2.0 * Math.PI * 62.5 * dt);
        let saw = 0.0;
        for (let h = 1; h <= 6; h++) {
          saw += Math.sin(2.0 * Math.PI * (250.0 * h) * dt) / Math.pow(h, 0.85);
        }
        out[n] = 0.52 * (envSub * sub + 0.65 * envSaw * saw);
      }
      return out;
    }

    /** Exact 10-stage 2nd-order All-Pass Cascade (|H(f)| = 1, matched to 8 kHz pole decay). */
    function applyAllPass10(beat, fs, f0 = 260.0, rAt8k = 0.90) {
      const N = beat.length;
      const r = Math.pow(rAt8k, 8000.0 / fs);
      const w0 = 2.0 * Math.PI * f0 / fs;
      const a1 = -2.0 * r * Math.cos(w0);
      const a2 = r * r;

      // Run 2 consecutive beats so circular tail wraps seamlessly with zero edge click
      let x = new Float64Array(N * 2);
      x.set(beat, 0);
      x.set(beat, N);
      let y = new Float64Array(N * 2);

      for (let stage = 0; stage < 10; stage++) {
        let x1 = 0, x2 = 0, y1 = 0, y2 = 0;
        for (let i = 0; i < N * 2; i++) {
          const x0 = x[i];
          const y0 = a2 * x0 + a1 * x1 + x2 - a1 * y1 - a2 * y2;
          y[i] = y0;
          x2 = x1; x1 = x0;
          y2 = y1; y1 = y0;
        }
        const tmp = x; x = y; y = tmp;
      }
      return x.slice(N, N * 2);
    }

    /** Exact zero-phase symmetric Blackman-windowed sinc lowpass (fc = 420 Hz) via FFT. */
    function applyLinearPhaseLowpass(beat, fs, fc = 420.0) {
      const N = beat.length;
      const halfM = Math.round(0.036 * fs); // 36 ms symmetric pre/post ringing span
      const L = 2 * halfM + 1;
      const h = new Float64Array(L);
      let hSum = 0.0;
      for (let i = 0; i < L; i++) {
        const k = i - halfM;
        const sinc = k === 0
          ? (2.0 * fc / fs)
          : Math.sin(2.0 * Math.PI * fc * k / fs) / (Math.PI * k);
        const w = 0.42 - 0.5 * Math.cos(2.0 * Math.PI * i / (L - 1))
                + 0.08 * Math.cos(4.0 * Math.PI * i / (L - 1));
        h[i] = sinc * w;
        hSum += h[i];
      }
      for (let i = 0; i < L; i++) h[i] /= hSum;

      // Circular FFT convolution on next power of 2 >= N + L
      let nFft = 1;
      while (nFft < N + L) nFft <<= 1;
      const fft = new Radix2FFT(nFft);
      const xRe = new Float64Array(nFft), xIm = new Float64Array(nFft);
      const hRe = new Float64Array(nFft), hIm = new Float64Array(nFft);

      for (let i = 0; i < N; i++) xRe[i] = beat[i];
      // Zero-phase placement: negative lags wrap to end of FFT buffer
      for (let i = 0; i < L; i++) {
        const k = i - halfM;
        const idx = (k + nFft) % nFft;
        hRe[idx] = h[i];
      }
      fft.transform(xRe, xIm, false);
      fft.transform(hRe, hIm, false);
      for (let i = 0; i < nFft; i++) {
        const re = xRe[i] * hRe[i] - xIm[i] * hIm[i];
        const im = xRe[i] * hIm[i] + xIm[i] * hRe[i];
        xRe[i] = re; xIm[i] = im;
      }
      fft.transform(xRe, xIm, true);

      // Fold tail back circularly onto length N
      const out = new Float64Array(N);
      for (let i = 0; i < N; i++) out[i] = xRe[i];
      for (let i = N; i < nFft; i++) out[i % N] += xRe[i];
      return out;
    }

    function buildSynthAudioBuffer(audioCtx, sceneId) {
      const fs = audioCtx.sampleRate;
      const rng = createMulberry32(202611 + sceneId * 7919);

      // Scenes 0, 1, 2: 6.0s seamless loop with randomized note-trigger phases every ~0.13s
      if (sceneId <= 2) {
        const nSegs = 46;
        const segSamples = Math.round(0.13 * fs);
        const totalN = nSegs * segSamples;
        const fadeN = Math.max(16, Math.round(0.006 * fs));
        const buf = audioCtx.createBuffer(1, totalN, fs);
        const ch = buf.getChannelData(0);

        const pA = new Float64Array(nSegs);
        const pB = new Float64Array(nSegs);
        const pC = new Float64Array(nSegs);
        for (let s = 0; s < nSegs; s++) {
          pA[s] = rng() * 2.0 * Math.PI;
          pB[s] = rng() * 2.0 * Math.PI;
          pC[s] = rng() * 2.0 * Math.PI;
        }

        const evalScene = (t, pa, pb, pc) => {
          if (sceneId === 0) {
            // 1. Clean Uncoupled Mix: 62.5 + 281.25 + 343.75 Hz independent phases
            return (
              0.36 * Math.cos(2.0 * Math.PI * 62.5 * t + pa) +
              0.36 * Math.cos(2.0 * Math.PI * 281.25 * t + pb) +
              0.28 * Math.cos(2.0 * Math.PI * 343.75 * t + pc)
            );
          } else if (sceneId === 1) {
            // 2. Odd Clipper (tanh(3.2 * x)): 93.75 + 156.25 Hz symmetric odd clipping
            const raw =
              0.68 * Math.cos(2.0 * Math.PI * 93.75 * t + pa) +
              0.58 * Math.cos(2.0 * Math.PI * 156.25 * t + pb);
            return 0.62 * Math.tanh(3.2 * raw);
          } else {
            // 3. Tube + Sub IMD (x + 0.45 * x^2): 62.5 Hz sub + 281.25 Hz mid
            const raw =
              0.62 * Math.cos(2.0 * Math.PI * 62.5 * t + pa) +
              0.52 * Math.cos(2.0 * Math.PI * 281.25 * t + pb);
            const dc = 0.45 * 0.5 * (0.62 * 0.62 + 0.52 * 0.52);
            return 0.55 * (raw + 0.45 * raw * raw - dc);
          }
        };

        const noiseStd = sceneId === 0 ? 0.075 : 0.045;
        for (let s = 0; s < nSegs; s++) {
          const prevS = (s - 1 + nSegs) % nSegs;
          const base = s * segSamples;
          for (let j = 0; j < segSamples; j++) {
            const idx = base + j;
            const t = idx / fs;
            let v = evalScene(t, pA[s], pB[s], pC[s]);
            if (j < fadeN) {
              const w = 0.5 - 0.5 * Math.cos(Math.PI * j / fadeN);
              const vPrev = evalScene(t, pA[prevS], pB[prevS], pC[prevS]);
              v = vPrev * (1.0 - w) + v * w;
            }
            ch[idx] = v + noiseStd * gaussNoise(rng);
          }
        }
        return buf;
      }

      // Scenes 3, 4, 5, 6: Exact 125 BPM (0.48s) kicks tiled x4 beats (1.92s)
      const dryBeat = buildDryKickBeat(fs, 0.48);
      let beat;
      if (sceneId === 3) {
        beat = dryBeat;
      } else if (sceneId === 4) {
        beat = applyAllPass10(dryBeat, fs, 260.0, 0.90);
      } else if (sceneId === 5) {
        // Exact time-reverse of dryBeat
        const N = dryBeat.length;
        beat = new Float64Array(N);
        for (let i = 0; i < N; i++) beat[i] = dryBeat[N - 1 - i];
      } else {
        beat = applyLinearPhaseLowpass(dryBeat, fs, 420.0);
      }

      const nBeats = 4;
      const N = beat.length;
      const buf = audioCtx.createBuffer(1, N * nBeats, fs);
      const ch = buf.getChannelData(0);
      for (let b = 0; b < nBeats; b++) {
        for (let i = 0; i < N; i++) {
          ch[b * N + i] = beat[i] + 0.012 * gaussNoise(rng);
        }
      }
      return buf;
    }

    // =========================================================================
    // 7. INLINE BLOB-URL AUDIOWORKLETPROCESSOR (+ ScriptProcessorNode Fallback)
    // =========================================================================
    const WORKLET_CODE = `
      class DecimatorTapProcessor extends AudioWorkletProcessor {
        constructor(options) {
          super();
          const opts = options.processorOptions || {};
          this.D = Math.max(1, opts.D || 6);
          const s = opts.sections || [];
          this.s0 = s[0] || { b0: 1, b1: 0, b2: 0, a1: 0, a2: 0 };
          this.s1 = s[1] || { b0: 1, b1: 0, b2: 0, a1: 0, a2: 0 };
          this.z01 = 0; this.z02 = 0;
          this.z11 = 0; this.z12 = 0;
          this.decimCount = 0;
          this.chunkSize = 64;
          this.outBuf = new Float32Array(this.chunkSize);
          this.outPos = 0;
        }

        process(inputs) {
          const input = inputs[0];
          if (!input || input.length === 0) return true;
          const nCh = input.length;
          const ch0 = input[0];
          const ch1 = nCh > 1 ? input[1] : null;
          const len = ch0.length;

          const s0 = this.s0, s1 = this.s1;
          let z01 = this.z01, z02 = this.z02;
          let z11 = this.z11, z12 = this.z12;
          let dc = this.decimCount;
          const D = this.D;

          for (let i = 0; i < len; i++) {
            const x = ch1 ? 0.5 * (ch0[i] + ch1[i]) : ch0[i];
            // Biquad 1 (Direct Form II Transposed)
            const y1 = s0.b0 * x + z01;
            z01 = s0.b1 * x - s0.a1 * y1 + z02;
            z02 = s0.b2 * x - s0.a2 * y1;
            // Biquad 2
            const y2 = s1.b0 * y1 + z11;
            z11 = s1.b1 * y1 - s1.a1 * y2 + z12;
            z12 = s1.b2 * y1 - s1.a2 * y2;

            dc++;
            if (dc >= D) {
              dc = 0;
              this.outBuf[this.outPos++] = y2;
              if (this.outPos >= this.chunkSize) {
                const copy = this.outBuf.slice();
                this.port.postMessage(copy, [copy.buffer]);
                this.outPos = 0;
              }
            }
          }
          this.z01 = z01; this.z02 = z02;
          this.z11 = z11; this.z12 = z12;
          this.decimCount = dc;
          return true;
        }
      }
      registerProcessor("decimator-tap-processor", DecimatorTapProcessor);
    `;

    // =========================================================================
    // 8. 2D OSCILLOSCOPE & 2ND-ORDER RTA RENDERERS
    // =========================================================================
    function resize2D(canvas) {
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      const rect = canvas.getBoundingClientRect();
      const w = Math.max(1, Math.round(rect.width * dpr));
      const h = Math.max(1, Math.round(rect.height * dpr));
      if (canvas.width !== w || canvas.height !== h) {
        canvas.width = w;
        canvas.height = h;
      }
      return { w, h, dpr };
    }

    function drawOscilloscope(canvas, engine) {
      const { w, h, dpr } = resize2D(canvas);
      const ctx = canvas.getContext("2d");
      ctx.fillStyle = "#06090f";
      ctx.fillRect(0, 0, w, h);

      // Subtle grid
      ctx.strokeStyle = "rgba(53, 76, 115, 0.28)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, h * 0.5); ctx.lineTo(w, h * 0.5);
      ctx.moveTo(w * 0.35, 0); ctx.lineTo(w * 0.35, h);
      ctx.stroke();

      // Linearize ring buffer (latest 1024 decimated samples = ~128 ms)
      const N = engine.scopeLen;
      const linear = new Float32Array(N);
      const wr = engine.scopeWrite;
      for (let i = 0; i < N; i++) {
        linear[i] = engine.scopeRing[(wr + i) & (N - 1)];
      }

      // Peak-centered trigger so pre-ringing sits to the LEFT and decay/chirp to the RIGHT
      const viewN = 512; // ~64 ms display window
      const preSamples = Math.round(viewN * 0.35);
      let peakIdx = N - viewN + preSamples;
      let peakVal = 0.0;
      const searchStart = preSamples;
      const searchEnd = N - (viewN - preSamples);
      for (let i = searchStart; i < searchEnd; i++) {
        const av = Math.abs(linear[i]);
        if (av > peakVal) {
          peakVal = av;
          peakIdx = i;
        }
      }
      const start = Math.max(0, Math.min(N - viewN, peakIdx - preSamples));
      const scale = peakVal > 0.01 ? (0.42 * h / peakVal) : (0.42 * h);

      // Draw waveform
      ctx.strokeStyle = "#22d3ee";
      ctx.lineWidth = 1.6 * dpr;
      ctx.beginPath();
      for (let i = 0; i < viewN; i++) {
        const x = (i / (viewN - 1)) * w;
        const y = h * 0.5 - linear[start + i] * scale;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Axis labels
      ctx.fillStyle = "rgba(159, 176, 201, 0.72)";
      ctx.font = `${10 * dpr}px ${getComputedStyle(document.documentElement).getPropertyValue("--mono")}`;
      const winMs = (viewN / engine.fsA) * 1000;
      ctx.fillText(`Window: ${winMs.toFixed(1)} ms | Peak: ${peakVal.toFixed(3)}`, 8 * dpr, 14 * dpr);
      ctx.fillText("Trigger", w * 0.35 + 4 * dpr, 14 * dpr);
    }

    function drawRTA(canvas, engine) {
      const { w, h, dpr } = resize2D(canvas);
      const ctx = canvas.getContext("2d");
      ctx.fillStyle = "#06090f";
      ctx.fillRect(0, 0, w, h);

      const bins = engine.rtaBins;
      const dbMin = -85, dbMax = -5;
      const mapY = db => {
        const norm = (db - dbMin) / (dbMax - dbMin);
        return h - Math.min(1, Math.max(0, norm)) * (h - 18 * dpr) - 10 * dpr;
      };

      // dB horizontal gridlines
      ctx.strokeStyle = "rgba(53, 76, 115, 0.25)";
      ctx.lineWidth = 1;
      ctx.fillStyle = "rgba(100, 120, 150, 0.75)";
      ctx.font = `${9.5 * dpr}px ui-monospace, monospace`;
      for (const db of [-20, -40, -60, -80]) {
        const y = mapY(db);
        ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
        ctx.fillText(`${db} dB`, 6 * dpr, y - 3 * dpr);
      }

      // Optional locked reference trace (dashed amber) to prove All-Pass / Reverse RTA invariance
      if (engine.rtaRefDb) {
        ctx.strokeStyle = "rgba(245, 158, 11, 0.88)";
        ctx.lineWidth = 1.5 * dpr;
        ctx.setLineDash([5 * dpr, 4 * dpr]);
        ctx.beginPath();
        for (let b = 1; b < bins; b++) {
          const x = ((b - 1) / (bins - 2)) * w;
          const y = mapY(engine.rtaRefDb[b]);
          if (b === 1) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.stroke();
        ctx.setLineDash([]);
      }

      // Live 2nd-order power spectrum fill + stroke
      ctx.beginPath();
      for (let b = 1; b < bins; b++) {
        const x = ((b - 1) / (bins - 2)) * w;
        const db = 10.0 * Math.log10(Math.max(engine.rtaPower[b], 1e-12));
        const y = mapY(db);
        if (b === 1) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.strokeStyle = "#38bdf8";
      ctx.lineWidth = 1.7 * dpr;
      ctx.stroke();

      ctx.lineTo(w, h);
      ctx.lineTo(0, h);
      ctx.closePath();
      ctx.fillStyle = "rgba(56, 189, 248, 0.14)";
      ctx.fill();

      // Frequency ticks
      const fNyq = engine.fsA * 0.5;
      ctx.fillStyle = "rgba(159, 176, 201, 0.72)";
      for (const frac of [0.25, 0.5, 0.75]) {
        const x = frac * w;
        ctx.fillText(`${Math.round(fNyq * frac)} Hz`, x - 16 * dpr, h - 2 * dpr);
      }
      if (engine.rtaRefDb) {
        ctx.fillStyle = "#fbbf24";
        ctx.fillText("REF LOCKED (dashed)", w - 125 * dpr, 14 * dpr);
      }
    }

    // =========================================================================
    // 9. MAIN APPLICATION CONTROLLER & WEB AUDIO GRAPH
    // =========================================================================
    const ui = {
      liveDot: document.getElementById("liveDot"),
      verdictBadge: document.getElementById("verdictBadge"),
      sysTelemetry: document.getElementById("sysTelemetry"),
      statusBanner: document.getElementById("statusBanner"),

      tabSynth: document.getElementById("tabSynth"),
      tabMic: document.getElementById("tabMic"),
      tabFile: document.getElementById("tabFile"),
      paneSynth: document.getElementById("paneSynth"),
      paneMic: document.getElementById("paneMic"),
      paneFile: document.getElementById("paneFile"),

      sceneGrid: document.getElementById("sceneGrid"),
      btnSynthToggle: document.getElementById("btnSynthToggle"),
      slVolume: document.getElementById("slVolume"),
      lblVolume: document.getElementById("lblVolume"),

      btnMicToggle: document.getElementById("btnMicToggle"),

      dropzone: document.getElementById("dropzone"),
      fileInput: document.getElementById("fileInput"),
      fileInfo: document.getElementById("fileInfo"),
      btnFilePlay: document.getElementById("btnFilePlay"),
      btnFileLoop: document.getElementById("btnFileLoop"),
      slFileSeek: document.getElementById("slFileSeek"),
      lblFileTime: document.getElementById("lblFileTime"),
      slFileVolume: document.getElementById("slFileVolume"),
      lblFileVolume: document.getElementById("lblFileVolume"),

      slTau: document.getElementById("slTau"),
      lblTau: document.getElementById("lblTau"),
      slGate: document.getElementById("slGate"),
      lblGate: document.getElementById("lblGate"),
      lblKeff: document.getElementById("lblKeff"),
      btnAsymShader: document.getElementById("btnAsymShader"),
      btnResetEma: document.getElementById("btnResetEma"),

      glBi: document.getElementById("glBi"),
      glTri: document.getElementById("glTri"),
      tagBiGL: document.getElementById("tagBiGL"),
      tagTriGL: document.getElementById("tagTriGL"),
      metaBi: document.getElementById("metaBi"),
      metaTri: document.getElementById("metaTri"),
      lblBiX: document.getElementById("lblBiX"),
      lblBiY: document.getElementById("lblBiY"),
      lblTriX: document.getElementById("lblTriX"),
      lblTriY: document.getElementById("lblTriY"),
      inspBi: document.getElementById("inspBi"),
      inspTri: document.getElementById("inspTri"),
      topBiSummary: document.getElementById("topBiSummary"),
      topTriSummary: document.getElementById("topTriSummary"),

      valPhiRev: document.getElementById("valPhiRev"),
      barPhiRev: document.getElementById("barPhiRev"),
      valDjs: document.getElementById("valDjs"),
      valImRms: document.getElementById("valImRms"),
      subImBias: document.getElementById("subImBias"),
      valCrest: document.getElementById("valCrest"),
      subCrestDb: document.getElementById("subCrestDb"),
      valHspec: document.getElementById("valHspec"),
      valPeaks: document.getElementById("valPeaks"),
      subPeaks: document.getElementById("subPeaks"),

      cvScope: document.getElementById("cvScope"),
      cvRta: document.getElementById("cvRta"),
      btnLockRta: document.getElementById("btnLockRta"),
      metaRta: document.getElementById("metaRta")
    };

    function showBanner(msg, isError = false) {
      if (!msg) {
        ui.statusBanner.className = "status-banner";
        ui.statusBanner.textContent = "";
        return;
      }
      ui.statusBanner.textContent = msg;
      ui.statusBanner.className = "status-banner show" + (isError ? " error" : "");
    }

    const engine = new RollingSpectralEngine(8000, 0.6, -55.0);
    const glBiRenderer  = new HeatmapRenderer(ui.glBi,  engine.biW,  engine.biH,  ui.tagBiGL);
    const glTriRenderer = new HeatmapRenderer(ui.glTri, engine.triW, engine.triH, ui.tagTriGL);

    const state = {
      audioCtx: null,
      analysisBus: null,
      monitorGain: null,
      zeroGain: null,
      tapNode: null,
      tapMode: "--",
      decimFactor: 6,
      fsA: 8000,

      activeTab: "synth",
      asymShader: false,

      // Synth state
      synthScene: 0,
      synthPlaying: false,
      synthSource: null,
      synthCache: new Map(),

      // Mic state
      micActive: false,
      micStream: null,
      micSource: null,

      // File player state
      fileBuffer: null,
      fileName: "",
      fileSource: null,
      filePlaying: false,
      fileLoop: true,
      fileOffset: 0,
      fileStartTime: 0,
      isSeeking: false
    };

    async function ensureAudioGraph() {
      if (state.audioCtx) {
        if (state.audioCtx.state === "suspended") {
          await state.audioCtx.resume();
        }
        return;
      }

      const AudioCtx = window.AudioContext || window.webkitAudioContext;
      if (!AudioCtx) {
        showBanner("Web Audio API is not supported in this browser.", true);
        throw new Error("No AudioContext");
      }

      const ctx = new AudioCtx();
      const fsNative = ctx.sampleRate;
      const D = Math.max(1, Math.round(fsNative / 8000));
      const fsA = fsNative / D;
      const sections = designButterworth4LP(fsNative, 0.42 * fsA);

      state.audioCtx = ctx;
      state.decimFactor = D;
      state.fsA = fsA;
      engine.configure(fsA, parseFloat(ui.slTau.value), parseFloat(ui.slGate.value));
      engine.reset();
      updateAxisLabels();

      // Two strict buses: analysisBus (never audible) and monitorGain (speakers)
      state.analysisBus = ctx.createGain();
      state.analysisBus.gain.value = 1.0;

      state.monitorGain = ctx.createGain();
      state.monitorGain.gain.value = parseFloat(ui.slVolume.value);
      state.monitorGain.connect(ctx.destination);

      state.zeroGain = ctx.createGain();
      state.zeroGain.gain.value = 0.0;
      state.zeroGain.connect(ctx.destination);

      // Try inline Blob-URL AudioWorkletProcessor first, fallback seamlessly to ScriptProcessorNode
      let workletOk = false;
      if (ctx.audioWorklet && typeof AudioWorkletNode !== "undefined") {
        let blobUrl = null;
        try {
          const blob = new Blob([WORKLET_CODE], { type: "application/javascript" });
          blobUrl = URL.createObjectURL(blob);
          await ctx.audioWorklet.addModule(blobUrl);
          const node = new AudioWorkletNode(ctx, "decimator-tap-processor", {
            numberOfInputs: 1,
            numberOfOutputs: 1,
            channelCount: 2,
            processorOptions: { D, sections }
          });
          node.port.onmessage = ev => {
            engine.pushDecimated(ev.data);
          };
          state.analysisBus.connect(node);
          node.connect(state.zeroGain);
          state.tapNode = node;
          state.tapMode = "Worklet";
          workletOk = true;
        } catch (err) {
          console.warn("AudioWorklet unavailable, switching to ScriptProcessor fallback:", err);
        } finally {
          if (blobUrl) URL.revokeObjectURL(blobUrl);
        }
      }

      if (!workletOk) {
        // Automatic ScriptProcessorNode fallback (works cleanly on file:// origins)
        const bufSize = 1024;
        const sp = ctx.createScriptProcessor(bufSize, 2, 1);
        const s0 = sections[0], s1 = sections[1];
        let z01 = 0, z02 = 0, z11 = 0, z12 = 0, dc = 0;
        const outChunk = new Float32Array(Math.ceil(bufSize / D) + 2);

        sp.onaudioprocess = ev => {
          const inp = ev.inputBuffer;
          const ch0 = inp.getChannelData(0);
          const ch1 = inp.numberOfChannels > 1 ? inp.getChannelData(1) : null;
          let outCount = 0;
          for (let i = 0; i < ch0.length; i++) {
            const x = ch1 ? 0.5 * (ch0[i] + ch1[i]) : ch0[i];
            const y1 = s0.b0 * x + z01;
            z01 = s0.b1 * x - s0.a1 * y1 + z02;
            z02 = s0.b2 * x - s0.a2 * y1;
            const y2 = s1.b0 * y1 + z11;
            z11 = s1.b1 * y1 - s1.a1 * y2 + z12;
            z12 = s1.b2 * y1 - s1.a2 * y2;
            dc++;
            if (dc >= D) {
              dc = 0;
              outChunk[outCount++] = y2;
            }
          }
          if (outCount > 0) {
            engine.pushDecimated(outChunk.subarray(0, outCount));
          }
          // Output remains silent
          ev.outputBuffer.getChannelData(0).fill(0);
        };
        state.analysisBus.connect(sp);
        sp.connect(state.zeroGain);
        state.tapNode = sp;
        state.tapMode = "ScriptProc";
      }

      if (ctx.state === "suspended") await ctx.resume();
    }

    function updateAxisLabels() {
      const df = engine.df;
      ui.lblBiY.textContent  = `f2 ↑ 0 .. ${Math.round(32 * df)} Hz`;
      ui.lblBiX.textContent  = `f1 → 0 .. ${Math.round(64 * df)} Hz (f1+f2 ≤ 64 bins)`;
      ui.lblTriY.textContent = `f2 ↑ 0 .. ${Math.round(18 * df)} Hz`;
      ui.lblTriX.textContent = `f1 → 0 .. ${Math.round(36 * df)} Hz (f1+f2+f3 ≤ 36 bins)`;
      ui.metaRta.textContent = `0 .. ${Math.round(engine.fsA * 0.5)} Hz (df=${df.toFixed(1)} Hz)`;
    }

    function stopAllSources() {
      stopSynth();
      stopMic();
      pauseFile(false);
    }

    // --- Synth Testbed Controls ---
    function stopSynth() {
      if (state.synthSource) {
        try { state.synthSource.stop(); state.synthSource.disconnect(); } catch (_) {}
        state.synthSource = null;
      }
      state.synthPlaying = false;
      ui.btnSynthToggle.textContent = "▶ Start Synth";
      ui.btnSynthToggle.className = "primary";
    }

    async function startSynth(sceneId = state.synthScene) {
      await ensureAudioGraph();
      stopMic();
      pauseFile(false);
      stopSynth();
      showBanner("");

      state.synthScene = sceneId;
      let buf = state.synthCache.get(sceneId);
      if (!buf) {
        buf = buildSynthAudioBuffer(state.audioCtx, sceneId);
        state.synthCache.set(sceneId, buf);
      }

      const src = state.audioCtx.createBufferSource();
      src.buffer = buf;
      src.loop = true;
      src.connect(state.analysisBus);
      src.connect(state.monitorGain);
      src.start();

      state.synthSource = src;
      state.synthPlaying = true;
      ui.btnSynthToggle.textContent = "■ Stop Synth";
      ui.btnSynthToggle.className = "danger";
    }

    // --- Live Microphone Controls (Strictly Never Routed to Speakers) ---
    function stopMic() {
      if (state.micSource) {
        try { state.micSource.disconnect(); } catch (_) {}
        state.micSource = null;
      }
      if (state.micStream) {
        for (const tr of state.micStream.getTracks()) tr.stop();
        state.micStream = null;
      }
      state.micActive = false;
      ui.btnMicToggle.textContent = "● Start Live Mic";
      ui.btnMicToggle.className = "primary";
    }

    async function startMic() {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        showBanner("Microphone API unavailable (requires HTTPS or localhost).", true);
        return;
      }
      try {
        await ensureAudioGraph();
        stopAllSources();
        showBanner("Requesting raw microphone access...");
        const stream = await navigator.mediaDevices.getUserMedia({
          audio: {
            echoCancellation: false,
            noiseSuppression: false,
            autoGainControl: false
          }
        });
        const src = state.audioCtx.createMediaStreamSource(stream);
        // CRITICAL: Route ONLY to analysisBus — NEVER to monitorGain / audioCtx.destination
        src.connect(state.analysisBus);
        state.micStream = stream;
        state.micSource = src;
        state.micActive = true;
        engine.reset();
        ui.btnMicToggle.textContent = "■ Stop Live Mic";
        ui.btnMicToggle.className = "danger";
        showBanner("Live microphone active (monitoring muted to prevent feedback).");
      } catch (err) {
        stopMic();
        showBanner(`Microphone error: ${err.message || err.name}`, true);
      }
    }

    // --- Drag-and-Drop & File Player Controls ---
    function fmtTime(sec) {
      if (!isFinite(sec) || sec < 0) return "0:00";
      const m = Math.floor(sec / 60);
      const s = Math.floor(sec % 60);
      return `${m}:${s < 10 ? "0" : ""}${s}`;
    }

    function getFilePlaybackPos() {
      if (!state.fileBuffer) return 0;
      const dur = state.fileBuffer.duration;
      if (!state.filePlaying || !state.audioCtx) return Math.min(dur, state.fileOffset);
      const elapsed = state.audioCtx.currentTime - state.fileStartTime;
      const pos = state.fileOffset + elapsed;
      return state.fileLoop ? (pos % dur) : Math.min(dur, pos);
    }

    function pauseFile(preservePos = true) {
      if (state.filePlaying && preservePos) {
        state.fileOffset = getFilePlaybackPos();
      }
      if (state.fileSource) {
        state.fileSource.onended = null;
        try { state.fileSource.stop(); state.fileSource.disconnect(); } catch (_) {}
        state.fileSource = null;
      }
      state.filePlaying = false;
      ui.btnFilePlay.textContent = "▶ Play";
      ui.btnFilePlay.className = "primary";
    }

    async function playFile(offsetSec = state.fileOffset) {
      if (!state.fileBuffer) return;
      await ensureAudioGraph();
      stopSynth();
      stopMic();
      pauseFile(false);
      showBanner("");

      const dur = state.fileBuffer.duration;
      const startPos = Math.max(0, Math.min(dur - 0.01, offsetSec));
      const src = state.audioCtx.createBufferSource();
      src.buffer = state.fileBuffer;
      src.loop = state.fileLoop;
      src.connect(state.analysisBus);
      src.connect(state.monitorGain);
      src.onended = () => {
        if (state.fileSource === src && !state.fileLoop) {
          state.filePlaying = false;
          state.fileOffset = 0;
          state.fileSource = null;
          ui.btnFilePlay.textContent = "▶ Play";
          ui.btnFilePlay.className = "primary";
        }
      };
      src.start(0, startPos);
      state.fileSource = src;
      state.fileOffset = startPos;
      state.fileStartTime = state.audioCtx.currentTime;
      state.filePlaying = true;
      ui.btnFilePlay.textContent = "❚❚ Pause";
      ui.btnFilePlay.className = "danger";
    }

    async function loadAudioFile(file) {
      if (!file) return;
      try {
        await ensureAudioGraph();
        showBanner(`Decoding "${file.name}"...`);
        const arrBuf = await file.arrayBuffer();
        const decoded = await state.audioCtx.decodeAudioData(arrBuf);
        pauseFile(false);
        state.fileBuffer = decoded;
        state.fileName = file.name;
        state.fileOffset = 0;

        ui.fileInfo.textContent =
          `${file.name}  |  ${decoded.sampleRate} Hz  |  ${decoded.numberOfChannels}ch  |  ${decoded.duration.toFixed(2)}s`;
        ui.btnFilePlay.disabled = false;
        ui.slFileSeek.disabled = false;
        ui.slFileSeek.value = "0";
        ui.lblFileTime.textContent = `0:00 / ${fmtTime(decoded.duration)}`;
        engine.reset();
        await playFile(0);
      } catch (err) {
        showBanner(`Could not decode audio file: ${err.message || err}`, true);
      }
    }

    // =========================================================================
    // 10. UI EVENT WIRING
    // =========================================================================
    function switchTab(tab) {
      state.activeTab = tab;
      ui.tabSynth.classList.toggle("active", tab === "synth");
      ui.tabMic.classList.toggle("active", tab === "mic");
      ui.tabFile.classList.toggle("active", tab === "file");
      ui.paneSynth.classList.toggle("active", tab === "synth");
      ui.paneMic.classList.toggle("active", tab === "mic");
      ui.paneFile.classList.toggle("active", tab === "file");
    }

    ui.tabSynth.addEventListener("click", () => switchTab("synth"));
    ui.tabMic.addEventListener("click",   () => switchTab("mic"));
    ui.tabFile.addEventListener("click",  () => switchTab("file"));

    ui.sceneGrid.addEventListener("click", async ev => {
      const btn = ev.target.closest(".scene-btn");
      if (!btn) return;
      const sid = parseInt(btn.dataset.scene, 10);
      for (const el of ui.sceneGrid.querySelectorAll(".scene-btn")) {
        el.classList.toggle("active", el === btn);
      }
      state.synthScene = sid;
      engine.reset();
      await startSynth(sid);
    });

    ui.btnSynthToggle.addEventListener("click", async () => {
      if (state.synthPlaying) stopSynth();
      else await startSynth(state.synthScene);
    });

    function syncVolume(val) {
      const v = parseFloat(val);
      ui.slVolume.value = String(v);
      ui.slFileVolume.value = String(v);
      const pct = `${Math.round(v * 100)}%`;
      ui.lblVolume.textContent = pct;
      ui.lblFileVolume.textContent = pct;
      if (state.monitorGain && state.audioCtx) {
        state.monitorGain.gain.setTargetAtTime(v, state.audioCtx.currentTime, 0.015);
      }
    }
    ui.slVolume.addEventListener("input", e => syncVolume(e.target.value));
    ui.slFileVolume.addEventListener("input", e => syncVolume(e.target.value));

    ui.btnMicToggle.addEventListener("click", async () => {
      if (state.micActive) { stopMic(); showBanner(""); }
      else await startMic();
    });

    // Dropzone & File input
    ui.dropzone.addEventListener("click", () => ui.fileInput.click());
    ui.fileInput.addEventListener("change", e => {
      if (e.target.files && e.target.files[0]) loadAudioFile(e.target.files[0]);
    });
    ui.dropzone.addEventListener("dragover", e => {
      e.preventDefault();
      ui.dropzone.classList.add("dragover");
    });
    ui.dropzone.addEventListener("dragleave", () => ui.dropzone.classList.remove("dragover"));
    ui.dropzone.addEventListener("drop", e => {
      e.preventDefault();
      ui.dropzone.classList.remove("dragover");
      if (e.dataTransfer.files && e.dataTransfer.files[0]) {
        loadAudioFile(e.dataTransfer.files[0]);
      }
    });

    ui.btnFilePlay.addEventListener("click", async () => {
      if (state.filePlaying) pauseFile(true);
      else await playFile(state.fileOffset);
    });
    ui.btnFileLoop.addEventListener("click", () => {
      state.fileLoop = !state.fileLoop;
      ui.btnFileLoop.classList.toggle("active", state.fileLoop);
      ui.btnFileLoop.textContent = state.fileLoop ? "Loop: ON" : "Loop: OFF";
      if (state.fileSource) state.fileSource.loop = state.fileLoop;
    });
    ui.slFileSeek.addEventListener("pointerdown", () => { state.isSeeking = true; });
    ui.slFileSeek.addEventListener("input", e => {
      if (!state.fileBuffer) return;
      const t = parseFloat(e.target.value) * state.fileBuffer.duration;
      ui.lblFileTime.textContent = `${fmtTime(t)} / ${fmtTime(state.fileBuffer.duration)}`;
    });
    ui.slFileSeek.addEventListener("change", async e => {
      state.isSeeking = false;
      if (!state.fileBuffer) return;
      const t = parseFloat(e.target.value) * state.fileBuffer.duration;
      state.fileOffset = t;
      if (state.filePlaying) await playFile(t);
    });

    // EMA & Shader controls
    ui.slTau.addEventListener("input", e => {
      const val = parseFloat(e.target.value);
      ui.lblTau.textContent = `${val.toFixed(2)} s`;
      engine.configure(state.fsA, val, parseFloat(ui.slGate.value));
    });
    ui.slGate.addEventListener("input", e => {
      const val = parseFloat(e.target.value);
      ui.lblGate.textContent = `${val.toFixed(0)} dBFS`;
      engine.configure(state.fsA, parseFloat(ui.slTau.value), val);
    });
    ui.btnAsymShader.addEventListener("click", () => {
      state.asymShader = !state.asymShader;
      ui.btnAsymShader.classList.toggle("active", state.asymShader);
      ui.btnAsymShader.textContent = state.asymShader
        ? "Im[B] Asymmetry Shader: ON"
        : "Im[B] Asymmetry Shader: OFF";
    });
    ui.btnResetEma.addEventListener("click", () => engine.reset());

    // RTA Reference Trace Lock
    ui.btnLockRta.addEventListener("click", () => {
      if (engine.rtaRefDb) {
        engine.rtaRefDb = null;
        ui.btnLockRta.classList.remove("active");
        ui.btnLockRta.textContent = "Lock Ref Trace";
      } else {
        const ref = new Float32Array(engine.rtaBins);
        for (let b = 0; b < engine.rtaBins; b++) {
          ref[b] = 10.0 * Math.log10(Math.max(engine.rtaPower[b], 1e-12));
        }
        engine.rtaRefDb = ref;
        ui.btnLockRta.classList.add("active");
        ui.btnLockRta.textContent = "Clear Ref Trace";
      }
    });

    // Pointer/Touch Hover Crosshair Inspector on both WebGL Heatmaps
    function bindHeatmapPointer(canvas, renderer) {
      const updatePos = ev => {
        const rect = canvas.getBoundingClientRect();
        const nx = (ev.clientX - rect.left) / Math.max(rect.width, 1);
        const ny = 1.0 - (ev.clientY - rect.top) / Math.max(rect.height, 1);
        if (nx >= 0 && nx < 1 && ny >= 0 && ny < 1) {
          renderer.hoverX = Math.floor(nx * renderer.gridW);
          renderer.hoverY = Math.floor(ny * renderer.gridH);
        } else {
          renderer.hoverX = -1;
          renderer.hoverY = -1;
        }
      };
      canvas.addEventListener("pointermove", updatePos);
      canvas.addEventListener("pointerdown", e => {
        if (e.pointerType !== "mouse") e.preventDefault();
        updatePos(e);
      });
      canvas.addEventListener("pointerleave", () => {
        renderer.hoverX = -1;
        renderer.hoverY = -1;
      });
    }
    bindHeatmapPointer(ui.glBi, glBiRenderer);
    bindHeatmapPointer(ui.glTri, glTriRenderer);

    // =========================================================================
    // 11. 60 FPS RENDER & TELEMETRY LOOP
    // =========================================================================
    function renderLoop() {
      const r = engine.computeReadout();

      // WebGL 2.0 Heatmaps
      glBiRenderer.render(engine.biTexRGBA, state.asymShader);
      glTriRenderer.render(engine.triTexRGBA, state.asymShader);

      // 2D Oscilloscope & RTA
      drawOscilloscope(ui.cvScope, engine);
      drawRTA(ui.cvRta, engine);

      // Header live status dot & verdict badge
      const isRunning = state.synthPlaying || state.micActive || state.filePlaying;
      ui.liveDot.className = "brand-dot" + (isRunning ? (r.gated ? " gated" : " live") : "");

      ui.verdictBadge.textContent = r.gated && isRunning ? "GATED (SILENCE)" : r.verdict;
      ui.verdictBadge.className = "verdict-badge " + (r.gated && isRunning ? "v-clean" : r.verdictClass);

      ui.sysTelemetry.textContent =
        `fs_a: ${Math.round(engine.fsA)} Hz (/${state.decimFactor}) | Tap: ${state.tapMode} | Hop: ${r.hopMs.toFixed(2)} ms`;
      ui.lblKeff.textContent = `K_eff(3): ${r.kEff3.toFixed(1)} | K_eff(4): ${r.kEff4.toFixed(1)}`;

      // Heatmap metadata & hover inspector bars
      ui.metaBi.textContent =
        `Bonf crit: ${r.critBi.toFixed(3)} | Hits: ${r.biHits} / ${engine.nTriads}`;
      ui.metaTri.textContent =
        `Bonf crit: ${r.critTri.toFixed(3)} | Hits: ${r.triHits} / ${engine.nQuartets}`;

      const hoverBi = glBiRenderer.hoverX >= 0
        ? engine.inspectBi(glBiRenderer.hoverX, glBiRenderer.hoverY)
        : null;
      if (hoverBi) {
        ui.inspBi.innerHTML =
          `<span><strong>(${hoverBi.f1.toFixed(0)}, ${hoverBi.f2.toFixed(0)} &rarr; ${hoverBi.f3.toFixed(0)} Hz)</strong> ` +
          `b&sup2;=<strong>${hoverBi.val.toFixed(3)}</strong> &psi;=<strong>${hoverBi.psiDeg >= 0 ? "+" : ""}${hoverBi.psiDeg.toFixed(0)}&deg;</strong> ` +
          `|sin&psi;|=${hoverBi.asym.toFixed(2)}</span>` +
          `<span class="sig-pill ${hoverBi.sig ? "yes" : "no"}">${hoverBi.sig ? "BONF SIG" : "NOISE"}</span>`;
      } else {
        const topStr = r.topTriad
          ? `Top: (${r.topTriad.f1.toFixed(0)}, ${r.topTriad.f2.toFixed(0)} -> ${r.topTriad.f3.toFixed(0)} Hz) b²=${r.topTriad.b2.toFixed(3)} ψ=${r.topTriad.psiDeg >= 0 ? "+" : ""}${r.topTriad.psiDeg.toFixed(0)}°`
          : "Top: --";
        ui.inspBi.innerHTML = `<span>Hover/touch heatmap to inspect triad</span><span><strong>${topStr}</strong></span>`;
      }

      const hoverTri = glTriRenderer.hoverX >= 0
        ? engine.inspectTri(glTriRenderer.hoverX, glTriRenderer.hoverY)
        : null;
      if (hoverTri) {
        ui.inspTri.innerHTML =
          `<span><strong>(${hoverTri.f1.toFixed(0)}, ${hoverTri.f2.toFixed(0)}, ${hoverTri.f3.toFixed(0)} &rarr; ${hoverTri.f4.toFixed(0)} Hz)</strong> ` +
          `t&sup2;=<strong>${hoverTri.val.toFixed(3)}</strong> &psi;=<strong>${hoverTri.psiDeg >= 0 ? "+" : ""}${hoverTri.psiDeg.toFixed(0)}&deg;</strong></span>` +
          `<span class="sig-pill ${hoverTri.sig ? "yes" : "no"}">${hoverTri.sig ? "BONF SIG" : "NOISE"}</span>`;
      } else {
        const topStr = r.topQuartet
          ? `Top: (${r.topQuartet.f1.toFixed(0)}, ${r.topQuartet.f2.toFixed(0)}, ${r.topQuartet.f3.toFixed(0)} -> ${r.topQuartet.f4.toFixed(0)} Hz) t²=${r.topQuartet.t2.toFixed(3)}`
          : "Top: --";
        ui.inspTri.innerHTML = `<span>Hover/touch heatmap to inspect quartet</span><span><strong>${topStr}</strong></span>`;
      }

      // Telemetry HUD cards
      ui.valPhiRev.textContent = `${r.phiRev >= 0 ? "+" : ""}${r.phiRev.toFixed(3)}`;
      const clampedPhi = Math.max(-1.5, Math.min(1.5, r.phiRev));
      const pct = Math.abs(clampedPhi / 1.5) * 50.0;
      ui.barPhiRev.style.width = `${pct.toFixed(1)}%`;
      ui.barPhiRev.style.left = clampedPhi >= 0 ? "50%" : `${(50.0 - pct).toFixed(1)}%`;
      ui.barPhiRev.style.background = clampedPhi >= 0 ? "var(--emerald)" : "var(--crimson)";

      ui.valDjs.textContent = r.dJs.toFixed(4);
      ui.valImRms.textContent = r.imRms.toFixed(4);
      ui.subImBias.textContent = `Im Polarity: ${r.imPolarity >= 0 ? "+" : ""}${r.imPolarity.toFixed(3)}`;

      ui.valCrest.textContent = r.crest.toFixed(2);
      const crestDb = r.crest > 1e-6 ? 20.0 * Math.log10(r.crest) : 0.0;
      ui.subCrestDb.textContent = `${crestDb.toFixed(1)} dB (Peak/RMS)`;

      ui.valHspec.textContent = r.hSpec.toFixed(3);
      ui.valPeaks.textContent = `${r.b2Max.toFixed(2)} / ${r.t2Max.toFixed(2)}`;
      ui.subPeaks.textContent = `b² max (${r.biHits} sig) / t² max (${r.triHits} sig)`;

      // Audio file scrubber update
      if (state.fileBuffer && state.filePlaying && !state.isSeeking) {
        const pos = getFilePlaybackPos();
        const dur = state.fileBuffer.duration;
        ui.slFileSeek.value = String(pos / Math.max(dur, 1e-6));
        ui.lblFileTime.textContent = `${fmtTime(pos)} / ${fmtTime(dur)}`;
      }

      requestAnimationFrame(renderLoop);
    }

    // Initialize labels & start live loop
    updateAxisLabels();
    requestAnimationFrame(renderLoop);
  })();
  </script>
</body>
</html>
