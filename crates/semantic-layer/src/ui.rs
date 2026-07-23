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
.tagline {
  font-size: 12px;
  color: #6b7280;
  letter-spacing: 0.03em;
}

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

/* ── Welcome state ── */
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
.welcome h1 {
  font-size: 26px;
  font-weight: 600;
  color: #111318;
}
.welcome p {
  font-size: 15px;
  color: #6b7280;
}
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
.chip:hover {
  border-color: #0052FF;
  background: #f0f5ff;
  color: #0052FF;
}

/* ── Messages ── */
.messages {
  display: flex;
  flex-direction: column;
  gap: 28px;
  padding-top: 24px;
}
.msg-q {
  font-size: 13px;
  color: #6b7280;
  margin-bottom: 6px;
}
.msg-a {
  font-size: 15px;
  line-height: 1.65;
  color: #111318;
}
.msg-a br { display: block; content: ''; margin-top: 4px; }

/* ── Loading indicator ── */
.loading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #6b7280;
  font-size: 14px;
}
.dot-pulse {
  display: flex;
  gap: 4px;
}
.dot-pulse span {
  width: 6px;
  height: 6px;
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

/* ── Dollar amounts bold ── */
.dollar { font-weight: 700; }

/* ── Divider between messages ── */
.msg-divider {
  border: none;
  border-top: 1px solid #e5e7eb;
  margin: 0;
}

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
const banner  = document.getElementById('banner');

input.addEventListener('keydown', e => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendQuestion(); } });

function fillQuestion(chip) {
  input.value = chip.textContent;
  input.focus();
}

function scrollToBottom() {
  scroll.scrollTop = scroll.scrollHeight;
}

function formatAnswer(text) {
  // Escape HTML first
  text = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // Severity badge replacements (case-insensitive)
  text = text.replace(/\bCRITICAL\b/gi,        '<span class="badge badge-critical">CRITICAL</span>');
  text = text.replace(/\bWATCH(?:\s+LIST)?\b/gi,'<span class="badge badge-watch">WATCH</span>');
  text = text.replace(/\bSTABLE\b/gi,           '<span class="badge badge-stable">STABLE</span>');
  text = text.replace(/\bPERFORMING WELL\b/gi,  '<span class="badge badge-stable">PERFORMING WELL</span>');

  // Bold dollar amounts
  text = text.replace(/(\$[\d,]+(?:\.\d+)?)/g, '<span class="dollar">$1</span>');

  // Newlines to line breaks
  text = text.replace(/\n/g, '<br>');

  return text;
}

async function sendQuestion() {
  const question = input.value.trim();
  if (!question) return;

  // Hide welcome, clear input, disable send
  welcome.style.display = 'none';
  input.value = '';
  sendBtn.disabled = true;

  // Add divider if there are existing messages
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

  // Loading indicator
  const loadDiv = document.createElement('div');
  loadDiv.className = 'msg-a loading';
  loadDiv.innerHTML = `
    <div class="dot-pulse"><span></span><span></span><span></span></div>
    Analyzing your data&hellip;
  `;
  msgs.appendChild(loadDiv);
  scrollToBottom();

  // Answer container (hidden until first token)
  const ansDiv = document.createElement('div');
  ansDiv.className = 'msg-a';
  ansDiv.style.display = 'none';
  msgs.appendChild(ansDiv);

  let rawText = '';

  try {
    const res = await fetch('/ask', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ question })
    });

    if (!res.ok) {
      throw new Error(`Server error ${res.status}`);
    }

    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      // Process complete SSE lines
      let newline;
      while ((newline = buffer.indexOf('\n')) !== -1) {
        const line = buffer.slice(0, newline).trim();
        buffer = buffer.slice(newline + 1);

        if (!line.startsWith('data:')) continue;
        const raw = line.slice(5).trimStart();
        if (raw === '[DONE]') break;

        // Tokens are JSON-encoded to preserve leading spaces
        let token;
        try { token = JSON.parse(raw).t; } catch { token = raw; }

        // Show answer div, hide loader on first token
        if (rawText === '') {
          loadDiv.remove();
          ansDiv.style.display = '';
        }
        rawText += token;
        ansDiv.innerHTML = formatAnswer(rawText);
        scrollToBottom();
      }
    }

    // Final format pass after stream ends
    if (rawText) {
      loadDiv.remove();
      ansDiv.style.display = '';
      ansDiv.innerHTML = formatAnswer(rawText);
    } else if (loadDiv.isConnected) {
      loadDiv.innerHTML = '<span style="color:#dc2626">No response received.</span>';
    }

  } catch (err) {
    loadDiv.remove();
    ansDiv.style.display = '';
    ansDiv.innerHTML = `<span style="color:#dc2626">Error: ${err.message}</span>`;
  }

  sendBtn.disabled = false;
  input.focus();
  scrollToBottom();
}

// Processing banner helpers (called externally if a pipeline runner is integrated)
function showBanner() {
  banner.classList.remove('hidden');
}
function hideBanner(msg) {
  if (msg) {
    banner.textContent = msg;
    setTimeout(() => { banner.style.opacity = '0'; setTimeout(() => banner.classList.add('hidden'), 500); }, 5000);
  } else {
    banner.classList.add('hidden');
  }
}
</script>
</body>
</html>"#;
