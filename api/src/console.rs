pub const CONSOLE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>DEZH Web Console</title>
<style>
  body { background:#0d1117; color:#c9d1d9; font-family: Consolas, monospace; margin:0; padding:20px; }
  h1 { color:#58a6ff; font-size:18px; }
  .panel { background:#161b22; border:1px solid #30363d; border-radius:6px; padding:12px; margin-bottom:16px; }
  .panel h2 { margin:0 0 8px 0; font-size:14px; color:#8b949e; }
  ul { list-style:none; padding:0; margin:0; }
  li { padding:4px 0; border-bottom:1px solid #21262d; }
  #output { background:#010409; padding:10px; height:220px; overflow-y:auto; white-space:pre-wrap; border-radius:4px; }
  input { background:#0d1117; border:1px solid #30363d; color:#c9d1d9; padding:6px; width:70%; font-family:inherit; }
  button { background:#238636; color:white; border:none; padding:6px 12px; cursor:pointer; border-radius:4px; }
</style>
</head>
<body>
  <h1>DEZH Web Console</h1>

  <div class="panel">
    <h2>Modules</h2>
    <ul id="modules-list"></ul>
  </div>

  <div class="panel">
    <h2>Core Services</h2>
    <ul id="core-services-list"></ul>
  </div>

  <div class="panel">
    <h2>Console</h2>
    <div id="output"></div>
    <br>
    <input id="cmd-input" placeholder="e.g. system" autofocus>
    <button onclick="runCommand()">Run</button>
  </div>

<script>
async function loadModules() {
  const res = await fetch('/api/modules');
  const modules = await res.json();
  const list = document.getElementById('modules-list');
  list.innerHTML = modules.length
    ? modules.map(m => `<li>${m.name} (${m.version})</li>`).join('')
    : '<li>No modules loaded</li>';
}

async function loadCoreServices() {
  const res = await fetch('/api/core-services');
  const services = await res.json();
  const list = document.getElementById('core-services-list');
  list.innerHTML = services.map(s => `<li>${s.name} — ${s.status}</li>`).join('');
}

async function runCommand() {
  const input = document.getElementById('cmd-input');
  const output = document.getElementById('output');
  const parts = input.value.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return;

  const name = parts[0];
  const args = parts.slice(1);

  output.textContent += `\nDEZH> ${input.value}\n`;

  try {
    const res = await fetch('/api/commands/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, args })
    });
    const data = await res.json();
    output.textContent += (data.output ?? data.error ?? 'No response') + '\n';
  } catch (e) {
    output.textContent += 'Request failed: ' + e + '\n';
  }

  output.scrollTop = output.scrollHeight;
  input.value = '';
}

document.getElementById('cmd-input').addEventListener('keydown', (e) => {
  if (e.key === 'Enter') runCommand();
});

loadModules();
loadCoreServices();
</script>
</body>
</html>"#;
