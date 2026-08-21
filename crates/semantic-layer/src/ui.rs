use axum::response::Html;

pub async fn index() -> Html<&'static str> {
    Html(HTML)
}

const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Optima Engine</title>
<style>
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: system-ui, -apple-system, sans-serif;
  background: #ffffff;
  color: #111318;
  height: 100dvh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ── Nav ── */
nav {
  flex-shrink: 0;
  height: 52px;
  background: #111318;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  z-index: 100;
}
.logo {
  font-family: 'Courier New', monospace;
  font-weight: bold;
  font-size: 16px;
  color: #ffffff;
  letter-spacing: 0.02em;
}
.logo-bang { color: #0052FF; }
.tagline { font-size: 12px; color: #6b7280; letter-spacing: 0.03em; }

/* ── Processing banner ── */
.banner {
  flex-shrink: 0;
  background: #fffbeb;
  border-bottom: 1px solid #fde68a;
  color: #92400e;
  font-size: 13px;
  padding: 8px 24px;
  text-align: center;
  transition: opacity 0.5s ease;
}
.banner.hidden { display: none; }

/* ── Scroll area ── */
.scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: 0 24px 16px;
}

/* ── Welcome ── */
.welcome {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  gap: 12px;
  padding: 40px 0;
}
.welcome h1 { font-size: 26px; font-weight: 600; color: #111318; }
.welcome p { font-size: 15px; color: #6b7280; }
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: center;
  margin-top: 8px;
  max-width: 600px;
}
.chip {
  border: 1px solid #e5e7eb;
  background: #ffffff;
  color: #111318;
  font-size: 13px;
  padding: 7px 14px;
  border-radius: 20px;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
  font-family: inherit;
}
.chip:hover { border-color: #0052FF; background: #f0f5ff; color: #0052FF; }

/* ── Messages ── */
.messages { display: flex; flex-direction: column; gap: 28px; padding-top: 24px; }
.msg-q { font-size: 13px; color: #6b7280; margin-bottom: 8px; }
.msg-divider { border: none; border-top: 1px solid #e5e7eb; margin: 0; }

/* ── Rotating loading messages ── */
.loading-state {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #6b7280;
  font-size: 14px;
  padding: 4px 0;
}
.dot-pulse { display: flex; gap: 4px; }
.dot-pulse span {
  width: 6px; height: 6px;
  background: #6b7280;
  border-radius: 50%;
  animation: pulse 1.2s ease-in-out infinite;
}
.dot-pulse span:nth-child(2) { animation-delay: 0.2s; }
.dot-pulse span:nth-child(3) { animation-delay: 0.4s; }
@keyframes pulse {
  0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
  40% { opacity: 1; transform: scale(1); }
}

/* ── Thinking panel ── */
.think-wrap { margin-bottom: 8px; }

.think-header {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
  color: #6b7280;
  font-size: 12px;
  padding: 4px 0;
}
.think-dot {
  width: 7px; height: 7px;
  background: #0052FF;
  border-radius: 50%;
}
.think-dot.pulsing { animation: tpulse 1s ease-in-out infinite; }
@keyframes tpulse {
  0%, 100% { opacity: 0.4; transform: scale(0.85); }
  50% { opacity: 1; transform: scale(1); }
}
.think-chevron {
  font-size: 10px;
  transition: transform 0.2s;
  display: none;
}
.think-chevron.visible { display: inline; }
.think-chevron.open { transform: rotate(90deg); }

.think-body {
  background: #f9fafb;
  border-left: 2px solid #0052FF;
  border-radius: 0 8px 8px 0;
  padding: 10px 14px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #6b7280;
  line-height: 1.55;
  max-height: 300px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
  transition: max-height 0.3s ease, opacity 0.3s ease;
}
.think-body.collapsed {
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
  opacity: 0;
  overflow: hidden;
}

/* ── Answer area ── */
.msg-a {
  font-size: 15px;
  line-height: 1.65;
  color: #111318;
}
.msg-a.hidden { display: none; }

/* ── Severity badges ── */
.badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 4px;
  letter-spacing: 0.04em;
  vertical-align: middle;
  margin: 0 2px;
}
.badge-critical { background: #fee2e2; color: #dc2626; }
.badge-watch    { background: #fffbeb; color: #d97706; }
.badge-stable   { background: #f0fdf4; color: #16a34a; }
.dollar         { font-weight: 700; }

/* ── Input bar ── */
.input-bar {
  flex-shrink: 0;
  border-top: 1px solid #e5e7eb;
  background: #ffffff;
  padding: 12px 24px;
  display: flex;
  gap: 10px;
  align-items: center;
}
.input-bar input {
  flex: 1;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 14px;
  font-family: inherit;
  color: #111318;
  outline: none;
  transition: border-color 0.15s;
}
.input-bar input:focus { border-color: #0052FF; }
.input-bar input::placeholder { color: #9ca3af; }
.input-bar button {
  background: #0052FF;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
}
.input-bar button:hover { background: #0041cc; }
.input-bar button:disabled { background: #93b4ff; cursor: not-allowed; }

/* ── Upload strip ── */
.upload-strip {
  flex-shrink: 0;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
  padding: 8px 24px;
  display: flex;
  align-items: center;
  gap: 10px;
}
.upload-strip label {
  font-size: 13px;
  font-weight: 600;
  color: #111318;
  white-space: nowrap;
  flex-shrink: 0;
}
.upload-file-input {
  font-size: 13px;
  font-family: inherit;
  color: #6b7280;
  min-width: 0;
  flex: 1;
}
.upload-btn {
  flex-shrink: 0;
  background: #0052FF;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  padding: 7px 16px;
  font-size: 13px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
}
.upload-btn:hover { background: #0041cc; }
.upload-btn:disabled { background: #93b4ff; cursor: not-allowed; }
.upload-status {
  font-size: 13px;
  flex-shrink: 0;
  white-space: nowrap;
  max-width: 260px;
  overflow: hidden;
  text-overflow: ellipsis;
}
.upload-status.success { color: #16a34a; }
.upload-status.error   { color: #dc2626; }

/* ── Mapping review panel ── */
.mapping-panel {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  max-height: 260px;
  border-bottom: 1px solid #e5e7eb;
  overflow: hidden;
}
.mapping-panel.hidden { display: none; }
.mapping-header {
  flex-shrink: 0;
  padding: 7px 24px;
  font-size: 12px;
  font-weight: 600;
  color: #6b7280;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}
.mapping-scroll { flex: 1; overflow-y: auto; min-height: 0; }
.mapping-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.mapping-table th {
  position: sticky;
  top: 0;
  background: #f9fafb;
  text-align: left;
  font-size: 11px;
  font-weight: 600;
  color: #6b7280;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 5px 24px;
  border-bottom: 1px solid #e5e7eb;
}
.mapping-table td {
  padding: 5px 24px;
  border-bottom: 1px solid #f3f4f6;
  vertical-align: middle;
}
.mapping-table tr:last-child td { border-bottom: none; }
.mapping-table tr.nomatch-row td { background: #fff5f5; }
.col-source { font-family: 'Courier New', monospace; font-size: 12px; color: #111318; width: 28%; white-space: nowrap; }
.col-select { width: 52%; }
.col-tier   { width: 20%; }
.canonical-select {
  width: 100%;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 4px 8px;
  font-size: 13px;
  font-family: inherit;
  color: #111318;
  background: #ffffff;
  cursor: pointer;
  outline: none;
}
.canonical-select:focus { border-color: #0052FF; }
.tier-badge {
  display: inline-block;
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  letter-spacing: 0.04em;
}
.tier-known     { background: #f0fdf4; color: #16a34a; }
.tier-heuristic { background: #fffbeb; color: #92400e; }
.tier-nomatch   { background: #fee2e2; color: #dc2626; }
.mapping-footer {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 24px;
  background: #f9fafb;
  border-top: 1px solid #e5e7eb;
}
.confirm-btn {
  background: #0052FF;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  padding: 7px 18px;
  font-size: 13px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
}
.confirm-btn:hover    { background: #0041cc; }
.confirm-btn:disabled { background: #93b4ff; cursor: not-allowed; }
.confirm-status { font-size: 13px; }
.confirm-status.success { color: #16a34a; }
.confirm-status.error   { color: #dc2626; }

/* ── Footer ── */
footer {
  flex-shrink: 0;
  text-align: center;
  font-size: 11px;
  color: #d1d5db;
  padding: 4px 0 8px;
}
</style>
</head>
<body>

<nav>
  <span class="logo">optima engine<span class="logo-bang">!</span></span>
  <span class="tagline">no data left the building</span>
</nav>

<div id="banner" class="banner hidden">
  Processing new data — your answers will reflect the latest export when complete.
</div>

<div class="upload-strip">
  <label for="erpFile">Upload ERP export</label>
  <input id="erpFile" class="upload-file-input" type="file" accept=".csv,.xlsx,.xls">
  <button class="upload-btn" id="uploadBtn" onclick="uploadFile()">Upload</button>
  <span id="uploadStatus" class="upload-status"></span>
</div>

<div id="mappingPanel" class="mapping-panel hidden">
  <div class="mapping-header">Review field mapping — correct any mismatches before confirming</div>
  <div class="mapping-scroll">
    <table class="mapping-table">
      <thead><tr>
        <th class="col-source">Source column</th>
        <th class="col-select">Maps to</th>
        <th class="col-tier"></th>
      </tr></thead>
      <tbody id="mappingBody"></tbody>
    </table>
  </div>
  <div class="mapping-footer">
    <button class="confirm-btn" id="confirmBtn" onclick="confirmMapping()">Confirm mapping</button>
    <span id="confirmStatus" class="confirm-status"></span>
  </div>
</div>

<div class="scroll-area" id="scrollArea">
  <div class="welcome" id="welcome">
    <h1>What do you want to know?</h1>
    <p>Ask anything about your ERP data.</p>
    <div class="chips">
      <button class="chip" onclick="fillQuestion(this)">What should our CFO be worried about?</button>
      <button class="chip" onclick="fillQuestion(this)">Where are we over budget?</button>
      <button class="chip" onclick="fillQuestion(this)">Which delivery routes are underperforming?</button>
      <button class="chip" onclick="fillQuestion(this)">What is our total logistics spend?</button>
    </div>
  </div>
  <div class="messages" id="messages"></div>
</div>

<div class="input-bar">
  <input id="questionInput" type="text" placeholder="Ask anything about your ERP data…" autocomplete="off">
  <button id="sendBtn" onclick="sendQuestion()">Send</button>
</div>

<footer>illustrative example &middot; no data left the building</footer>

<script>
const input   = document.getElementById('questionInput');
const sendBtn = document.getElementById('sendBtn');
const msgs    = document.getElementById('messages');
const welcome = document.getElementById('welcome');
const scroll  = document.getElementById('scrollArea');

input.addEventListener('keydown', e => {
  if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendQuestion(); }
});

function fillQuestion(chip) { input.value = chip.textContent; input.focus(); }
function scrollToBottom() { scroll.scrollTop = scroll.scrollHeight; }

// ── Format answer text ──────────────────────────────────────────────────────
function formatAnswer(text) {
  text = text.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
  text = text.replace(/\bCRITICAL\b/gi,       '<span class="badge badge-critical">CRITICAL</span>');
  text = text.replace(/\bWATCH(?:\s+LIST)?\b/gi,'<span class="badge badge-watch">WATCH</span>');
  text = text.replace(/\bSTABLE\b/gi,          '<span class="badge badge-stable">STABLE</span>');
  text = text.replace(/\bPERFORMING WELL\b/gi, '<span class="badge badge-stable">PERFORMING WELL</span>');
  text = text.replace(/(\$[\d,]+(?:\.\d+)?)/g, '<span class="dollar">$1</span>');
  text = text.replace(/\n/g, '<br>');
  return text;
}

// ── Rotating loading messages ────────────────────────────────────────────────
const ROTATE_MSGS = [
  'Reading your ERP data…',
  'Analyzing findings…',
  'Preparing your answer…'
];

function makeLoadingDiv() {
  const d = document.createElement('div');
  d.className = 'loading-state';
  d.innerHTML = `
    <div class="dot-pulse"><span></span><span></span><span></span></div>
    <span class="load-text">${ROTATE_MSGS[0]}</span>
  `;
  return d;
}

function startRotating(loadDiv) {
  const span = loadDiv.querySelector('.load-text');
  const t1 = setTimeout(() => { span.textContent = ROTATE_MSGS[1]; }, 3000);
  const t2 = setTimeout(() => { span.textContent = ROTATE_MSGS[2]; }, 8000);
  loadDiv._timers = [t1, t2];
}

function stopRotating(loadDiv) {
  (loadDiv._timers || []).forEach(clearTimeout);
}

// ── Think panel helpers ──────────────────────────────────────────────────────
function makeThinkWrap() {
  const wrap = document.createElement('div');
  wrap.className = 'think-wrap';
  wrap.innerHTML = `
    <div class="think-header" onclick="toggleThink(this)">
      <span class="think-dot pulsing"></span>
      <span class="think-label">Thinking…</span>
      <span class="think-chevron">›</span>
    </div>
    <div class="think-body"></div>
  `;
  return wrap;
}

function toggleThink(header) {
  const body = header.nextElementSibling;
  const chevron = header.querySelector('.think-chevron');
  body.classList.toggle('collapsed');
  chevron.classList.toggle('open');
}

function collapseThink(wrap, elapsedSecs) {
  const dot    = wrap.querySelector('.think-dot');
  const label  = wrap.querySelector('.think-label');
  const chev   = wrap.querySelector('.think-chevron');
  const body   = wrap.querySelector('.think-body');

  dot.classList.remove('pulsing');
  label.textContent = `Thought for ${elapsedSecs}s ›`;
  chev.classList.add('visible');
  body.classList.add('collapsed');
}

// ── Main send function ───────────────────────────────────────────────────────
async function sendQuestion() {
  const question = input.value.trim();
  if (!question) return;

  welcome.style.display = 'none';
  input.value = '';
  sendBtn.disabled = true;

  if (msgs.children.length > 0) {
    const hr = document.createElement('hr');
    hr.className = 'msg-divider';
    msgs.appendChild(hr);
  }

  // Question label
  const qDiv = document.createElement('div');
  qDiv.className = 'msg-q';
  qDiv.textContent = question;
  msgs.appendChild(qDiv);

  // Rotating loading indicator (shown until first token arrives)
  const loadDiv = makeLoadingDiv();
  msgs.appendChild(loadDiv);
  startRotating(loadDiv);
  scrollToBottom();

  // Think panel (hidden until thinking_start arrives)
  const thinkWrap = makeThinkWrap();
  thinkWrap.style.display = 'none';
  msgs.appendChild(thinkWrap);
  const thinkBody = thinkWrap.querySelector('.think-body');

  // Answer container
  const ansDiv = document.createElement('div');
  ansDiv.className = 'msg-a hidden';
  msgs.appendChild(ansDiv);

  let rawAnswer = '';
  let sawFirstToken = false;

  function onFirstToken() {
    if (sawFirstToken) return;
    sawFirstToken = true;
    stopRotating(loadDiv);
    loadDiv.remove();
  }

  try {
    const res = await fetch('/ask', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ question })
    });

    if (!res.ok) throw new Error(`Server error ${res.status}`);

    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      let nl;
      while ((nl = buffer.indexOf('\n')) !== -1) {
        const line = buffer.slice(0, nl).trim();
        buffer = buffer.slice(nl + 1);
        if (!line.startsWith('data:')) continue;

        const raw = line.slice(5).trimStart();
        if (raw === '[DONE]') break; // legacy fallback

        let ev;
        try { ev = JSON.parse(raw); } catch { continue; }

        switch (ev.type) {
          case 'thinking_start':
            onFirstToken();
            thinkWrap.style.display = '';
            scrollToBottom();
            break;

          case 'thinking':
            if (ev.t) {
              thinkBody.textContent += ev.t;
              thinkBody.scrollTop = thinkBody.scrollHeight;
              scrollToBottom();
            }
            break;

          case 'thinking_done':
            collapseThink(thinkWrap, ev.elapsed ?? 0);
            scrollToBottom();
            break;

          case 'answer':
          case 'direct':
            onFirstToken();
            if (ev.type === 'direct') thinkWrap.style.display = 'none';
            if (ev.t) {
              rawAnswer += ev.t;
              ansDiv.classList.remove('hidden');
              ansDiv.innerHTML = formatAnswer(rawAnswer);
              scrollToBottom();
            }
            break;

          case 'done':
            // Finalise
            if (rawAnswer) ansDiv.innerHTML = formatAnswer(rawAnswer);
            break;
        }
      }
    }

  } catch (err) {
    stopRotating(loadDiv);
    loadDiv.remove();
    thinkWrap.style.display = 'none';
    ansDiv.classList.remove('hidden');
    ansDiv.innerHTML = `<span style="color:#dc2626">Error: ${err.message}</span>`;
  }

  sendBtn.disabled = false;
  input.focus();
  scrollToBottom();
}

// ── Upload handler ───────────────────────────────────────────────────────────
async function uploadFile() {
  const fileInput = document.getElementById('erpFile');
  const statusEl  = document.getElementById('uploadStatus');
  const uploadBtn = document.getElementById('uploadBtn');

  if (!fileInput.files.length) {
    statusEl.className   = 'upload-status error';
    statusEl.textContent = 'Select a file first.';
    return;
  }

  const formData = new FormData();
  formData.append('file', fileInput.files[0]);

  uploadBtn.disabled   = true;
  statusEl.className   = 'upload-status';
  statusEl.textContent = 'Uploading…';

  try {
    const res  = await fetch('/ingest/upload', { method: 'POST', body: formData });
    const json = await res.json();

    if (res.ok) {
      const size = json.bytes < 1024
        ? `${json.bytes} B`
        : `${(json.bytes / 1024).toFixed(1)} KB`;
      statusEl.className   = 'upload-status success';
      statusEl.textContent = `✓ ${json.filename} saved (${size})`;
      fileInput.value = '';
      if (json.mapping && json.mapping.length > 0) renderMapping(json.mapping);
    } else {
      statusEl.className   = 'upload-status error';
      statusEl.textContent = `Error: ${JSON.stringify(json)}`;
    }
  } catch (err) {
    statusEl.className   = 'upload-status error';
    statusEl.textContent = `Error: ${err.message}`;
  }

  uploadBtn.disabled = false;
}

// ── Mapping review table ─────────────────────────────────────────────────────
const CANONICAL_FIELDS = [
  'order_id', 'customer_id', 'material_id', 'net_value', 'estimated_cost',
  'currency', 'distribution_channel', 'sales_org', 'industry', 'region_group',
  'department', 'fiscal_year', 'actual_cost', 'budget_amount', 'budget_variance',
  'route', 'transport_type', 'days_late', 'freight_cost_usd', 'delivery_id',
];

function renderMapping(mapping) {
  const tbody = document.getElementById('mappingBody');
  tbody.innerHTML = '';

  mapping.forEach(row => {
    const tr = document.createElement('tr');
    if (row.tier === 'NoMatch') tr.classList.add('nomatch-row');

    const tdSrc = document.createElement('td');
    tdSrc.className = 'col-source';
    tdSrc.textContent = row.header;
    tr.appendChild(tdSrc);

    const tdSel = document.createElement('td');
    tdSel.className = 'col-select';
    const sel = document.createElement('select');
    sel.className = 'canonical-select';
    sel.dataset.header = row.header;
    const notUsed = document.createElement('option');
    notUsed.value = '';
    notUsed.textContent = '— Not used —';
    if (!row.canonical) notUsed.selected = true;
    sel.appendChild(notUsed);
    CANONICAL_FIELDS.forEach(field => {
      const opt = document.createElement('option');
      opt.value = field;
      opt.textContent = field;
      if (field === row.canonical) opt.selected = true;
      sel.appendChild(opt);
    });
    tdSel.appendChild(sel);
    tr.appendChild(tdSel);

    const tdTier = document.createElement('td');
    tdTier.className = 'col-tier';
    const badge = document.createElement('span');
    badge.className = 'tier-badge tier-' + row.tier.toLowerCase();
    badge.textContent = row.tier;
    tdTier.appendChild(badge);
    tr.appendChild(tdTier);

    tbody.appendChild(tr);
  });

  document.getElementById('mappingPanel').classList.remove('hidden');
  document.getElementById('confirmStatus').textContent = '';
  document.getElementById('confirmStatus').className   = 'confirm-status';
}

async function confirmMapping() {
  const confirmBtn    = document.getElementById('confirmBtn');
  const confirmStatus = document.getElementById('confirmStatus');
  const selects       = document.querySelectorAll('.canonical-select');

  const body = Array.from(selects).map(sel => ({
    header:    sel.dataset.header,
    canonical: sel.value || null,
  }));

  confirmBtn.disabled       = true;
  confirmStatus.className   = 'confirm-status';
  confirmStatus.textContent = 'Saving…';

  try {
    const res  = await fetch('/ingest/confirm-mapping', {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify(body),
    });
    const json = await res.json();
    if (res.ok) {
      confirmStatus.className   = 'confirm-status success';
      confirmStatus.textContent = `✓ Mapping saved (${json.entries} fields)`;
    } else {
      confirmStatus.className   = 'confirm-status error';
      confirmStatus.textContent = `Error: ${JSON.stringify(json)}`;
    }
  } catch (err) {
    confirmStatus.className   = 'confirm-status error';
    confirmStatus.textContent = `Error: ${err.message}`;
  }

  confirmBtn.disabled = false;
}

function showBanner() { document.getElementById('banner').classList.remove('hidden'); }
function hideBanner(msg) {
  const b = document.getElementById('banner');
  if (msg) {
    b.textContent = msg;
    setTimeout(() => { b.style.opacity='0'; setTimeout(()=>b.classList.add('hidden'),500); }, 5000);
  } else { b.classList.add('hidden'); }
}
</script>
</body>
</html>"#;
