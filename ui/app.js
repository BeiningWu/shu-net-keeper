function togglePw(inputId, btn) {
  const input = document.getElementById(inputId);
  const isHidden = input.type === 'password';
  input.type = isHidden ? 'text' : 'password';
  btn.querySelector('.eye-show').style.display = isHidden ? 'none' : '';
  btn.querySelector('.eye-hide').style.display = isHidden ? '' : 'none';
}

(async function () {
  const { invoke } = window.__TAURI__.core;
  const { listen }  = window.__TAURI__.event;

  // ── SMTP toggle ──────────────────────────────────────────────────────────

  const smtpEnabledBox = document.getElementById("smtp-enabled");
  const smtpFields     = document.getElementById("smtp-fields");

  smtpEnabledBox.addEventListener("change", () => {
    smtpFields.classList.toggle("hidden", !smtpEnabledBox.checked);
  });

  // ── Load config ──────────────────────────────────────────────────────────

  async function loadConfig() {
    try {
      const cfg = await invoke("get_config");
      if (!cfg) return;
      document.getElementById("username").value = cfg.username ?? "";
      document.getElementById("password").value = cfg.password ?? "";
      document.getElementById("interval").value = cfg.interval ?? 600;
      smtpEnabledBox.checked = cfg.smtp_enabled ?? false;
      smtpFields.classList.toggle("hidden", !smtpEnabledBox.checked);
      if (cfg.smtp) {
        document.getElementById("smtp-server").value   = cfg.smtp.server   ?? "";
        document.getElementById("smtp-port").value     = cfg.smtp.port     ?? "";
        document.getElementById("smtp-sender").value   = cfg.smtp.sender   ?? "";
        document.getElementById("smtp-password").value = cfg.smtp.password ?? "";
        document.getElementById("smtp-receiver").value = cfg.smtp.receiver ?? "";
      }
    } catch (e) {
      console.error("load config failed:", e);
    }
  }

  // ── Save config ──────────────────────────────────────────────────────────

  document.getElementById("btn-save").addEventListener("click", async () => {
    const smtpEnabled = smtpEnabledBox.checked;
    const cfg = {
      username:     document.getElementById("username").value.trim(),
      password:     document.getElementById("password").value,
      interval:     parseInt(document.getElementById("interval").value, 10) || 600,
      smtp_enabled: smtpEnabled,
      smtp: smtpEnabled ? {
        server:   document.getElementById("smtp-server").value.trim()   || null,
        port:     parseInt(document.getElementById("smtp-port").value, 10) || null,
        sender:   document.getElementById("smtp-sender").value.trim()   || null,
        password: document.getElementById("smtp-password").value        || null,
        receiver: document.getElementById("smtp-receiver").value.trim() || null,
      } : null,
    };

    const msg = document.getElementById("save-msg");
    try {
      await invoke("save_config", { config: cfg });
      msg.textContent = "✓ 已保存";
      msg.className = "save-msg show";
      setTimeout(() => msg.classList.remove("show"), 2000);
    } catch (e) {
      msg.textContent = "✗ " + String(e);
      msg.className = "save-msg show error";
    }
  });

  // ── Status ───────────────────────────────────────────────────────────────

  function applyStatus(s) {
    const dot      = document.getElementById("conn-dot");
    const label    = document.getElementById("conn-label");
    const ip       = document.getElementById("conn-ip");
    const badge    = document.getElementById("status-badge");
    const errBox   = document.getElementById("error-box");
    const btnStart = document.getElementById("btn-start");
    const btnStop  = document.getElementById("btn-stop");

    document.getElementById("metric-logins").textContent     = s.login_count ?? 0;
    document.getElementById("metric-last-check").textContent = s.last_check  ?? "—";

    if (!s.running) {
      dot.className   = "conn-dot dot-idle";
      label.textContent = "未运行";
      ip.textContent    = "—";
      badge.textContent = "未运行";
      badge.className   = "badge badge-idle";
      btnStart.classList.remove("hidden");
      btnStop.classList.add("hidden");
      errBox.classList.add("hidden");
    } else if (s.connected) {
      dot.className   = "conn-dot dot-connected";
      label.textContent = "已连接";
      ip.textContent    = s.ip ? "IP: " + s.ip : "—";
      badge.textContent = "运行中";
      badge.className   = "badge badge-running";
      btnStart.classList.add("hidden");
      btnStop.classList.remove("hidden");
      errBox.classList.add("hidden");
    } else {
      dot.className   = "conn-dot dot-error";
      label.textContent = "未连接 / 重试中...";
      ip.textContent    = s.last_error ? "错误" : "—";
      badge.textContent = "运行中";
      badge.className   = "badge badge-running";
      btnStart.classList.add("hidden");
      btnStop.classList.remove("hidden");
      if (s.last_error) {
        errBox.textContent = s.last_error;
        errBox.classList.remove("hidden");
      } else {
        errBox.classList.add("hidden");
      }
    }
  }

  async function refreshStatus() {
    try { applyStatus(await invoke("get_status")); } catch (_) {}
  }

  // ── Daemon controls ──────────────────────────────────────────────────────

  document.getElementById("btn-start").addEventListener("click", async () => {
    try {
      await invoke("start_daemon");
    } catch (e) {
      alert("启动失败: " + String(e));
    }
  });

  document.getElementById("btn-stop").addEventListener("click", async () => {
    await invoke("stop_daemon");
  });

  // ── Logs ─────────────────────────────────────────────────────────────────

  const logContainer = document.getElementById("log-container");
  let hasLogs = false;

  function appendLog(line) {
    if (!hasLogs) {
      logContainer.innerHTML = "";
      hasLogs = true;
    }
    const div = document.createElement("div");
    div.className = "log-line" + classifyLog(line);
    div.textContent = line;
    logContainer.appendChild(div);
    if (document.getElementById("auto-scroll").checked) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }

  function classifyLog(line) {
    if (line.includes("✓"))                          return " success";
    if (line.includes("✗") || line.includes("失败")) return " error";
    if (line.includes("警告") || line.includes("warn")) return " warn";
    return " info";
  }

  async function loadLogs() {
    try {
      const logs = await invoke("get_logs");
      if (logs.length > 0) {
        logContainer.innerHTML = "";
        hasLogs = true;
        logs.forEach(appendLog);
      }
    } catch (_) {}
  }

  document.getElementById("btn-clear-logs").addEventListener("click", () => {
    logContainer.innerHTML = '<div class="log-placeholder">暂无日志</div>';
    hasLogs = false;
  });

  // ── Real-time events ─────────────────────────────────────────────────────

  await listen("status-update", (e) => applyStatus(e.payload));
  await listen("log-entry",     (e) => appendLog(e.payload));

  // ── Init ─────────────────────────────────────────────────────────────────

  await loadConfig();
  await refreshStatus();
  await loadLogs();

  setInterval(refreshStatus, 5000);
})();
