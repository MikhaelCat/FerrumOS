// Unified Feedback & Timed Status Helper (Auto-clear after duration)
function setTimedStatus(elementOrId, text, type = "success", duration = 3500) {
    const el = typeof elementOrId === "string" ? document.getElementById(elementOrId) : elementOrId;
    if (!el) return;
    if (el._statusTimer) {
        clearTimeout(el._statusTimer);
        el._statusTimer = null;
    }
    el.textContent = text;
    if (type === "success") {
        el.className = "status-badge status-badge-success align-self-center";
    } else if (type === "error") {
        el.className = "status-badge status-badge-error align-self-center";
    } else if (type === "info" || type === "loading") {
        el.className = "status-badge status-badge-info align-self-center";
    } else {
        el.className = "status-badge align-self-center";
    }
    if (duration > 0) {
        el._statusTimer = setTimeout(() => {
            el.textContent = "";
            el.className = "align-self-center";
            el._statusTimer = null;
        }, duration);
    }
}

// FerrumOS Modern Client Application (Zero Dependencies, Ultra Fast)
document.addEventListener("DOMContentLoaded", () => {
    initAuthManager();
    initNavigation();
    initDashboard();
    initProxyManager();
    initZapretManager();
    initGameBoostManager();
    initWifiManager();
    initRadarManager();
    initNetworkManager();
    initToolsManager();
    initSystemManager();
    initFirmwareUpgrade();
    startStatsPolling();
});

// Navigation & Tab Switching (Zero Reloads)
function initNavigation() {
    const tabs = {
        dashboard: { title: "Дашборд и мониторинг", subtitle: "Состояние аппаратных ресурсов и сетевой активности" },
        proxy: { title: "VLESS Reality & Hysteria 2", subtitle: "1-Click подключение и умный обход блокировок (РФ)" },
        zapret: { title: "Zapret DPI Bypass", subtitle: "Прямая локальная десинхронизация ТСПУ для YouTube 4K и Discord" },
        gameboost: { title: "ROG Game Boost & SQM QoS", subtitle: "Приоритезация трафика, сверхнизкий пинг и устранение Bufferbloat" },
        wifi: { title: "Беспроводная сеть Wi-Fi", subtitle: "Настройки 2.4 GHz (MT7620) и 5 GHz (MT7610E AC)" },
        radar: { title: "Wi-Fi Радар & Анализ эфира", subtitle: "Анализ радиопомех и автоматический выбор чистейшего канала (ACS)" },
        network: { title: "Сеть & Маршрутизация", subtitle: "Параметры WAN, коммутатора, LAN и DNS" },
        tools: { title: "Инструменты & WoL", subtitle: "Удаленный запуск ПК (Magic Packet), Watchdog, DDNS и диагностика" },
        system: { title: "Система & Гарантии защиты", subtitle: "Аппаратная защита калибровок MTD2 и управление питанием" },
    };

    document.querySelectorAll(".nav-item").forEach(button => {
        button.addEventListener("click", () => {
            const target = button.getAttribute("data-tab");
            document.querySelectorAll(".nav-item").forEach(b => b.classList.remove("active"));
            document.querySelectorAll(".tab-pane").forEach(p => p.classList.remove("active"));

            button.classList.add("active");
            const pane = document.getElementById(`tab-${target}`);
            if (pane) pane.classList.add("active");

            if (tabs[target]) {
                document.getElementById("page-title").textContent = tabs[target].title;
                document.getElementById("page-subtitle").textContent = tabs[target].subtitle;
            }

            try {
                localStorage.setItem("ferrum_active_tab", target);
                history.replaceState(null, null, `#${target}`);
            } catch (e) {}
        });
    });

    // Restore previously active tab on page refresh
    try {
        const savedTab = window.location.hash.slice(1) || localStorage.getItem("ferrum_active_tab") || "dashboard";
        const initialBtn = document.querySelector(`.nav-item[data-tab="${savedTab}"]`);
        if (initialBtn) {
            initialBtn.click();
        }
    } catch (e) {}

    document.getElementById("btn-refresh")?.addEventListener("click", () => {
        fetchSystemStatus();
        fetchInterfaces();
    });
}

// Live Traffic Canvas Chart
let rxHistory = new Array(30).fill(0);
let txHistory = new Array(30).fill(0);
let prevRx = 0, prevTx = 0;

function drawTrafficChart() {
    const canvas = document.getElementById("traffic-chart");
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    const width = canvas.width = canvas.parentElement.clientWidth;
    const height = canvas.height = 140;

    ctx.clearRect(0, 0, width, height);

    const maxVal = Math.max(...rxHistory, ...txHistory, 50);

    // Draw RX (Cyan)
    ctx.beginPath();
    ctx.strokeStyle = "#06b6d4";
    ctx.lineWidth = 2;
    for (let i = 0; i < rxHistory.length; i++) {
        const x = (i / (rxHistory.length - 1)) * width;
        const y = height - (rxHistory[i] / maxVal) * (height - 20) - 10;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    }
    ctx.stroke();

    // Draw TX (Purple)
    ctx.beginPath();
    ctx.strokeStyle = "#8b5cf6";
    ctx.lineWidth = 2;
    for (let i = 0; i < txHistory.length; i++) {
        const x = (i / (txHistory.length - 1)) * width;
        const y = height - (txHistory[i] / maxVal) * (height - 20) - 10;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    }
    ctx.stroke();
}

// System Status Polling
async function fetchSystemStatus() {
    try {
        const res = await fetch("/api/v1/system/status");
        if (!res.ok) return;
        const data = await res.json();

        // Topbar
        const cpuPercent = Math.min(Math.round(data.cpu_load * 100), 100);
        document.getElementById("top-cpu").textContent = `${cpuPercent}%`;
        const ramUsedMb = Math.round((data.ram_total_kb - data.ram_free_kb) / 1024);
        document.getElementById("top-ram").textContent = `${ramUsedMb} / 64 MB`;

        // Dashboard
        document.getElementById("dash-cpu-load").textContent = `Load avg: ${data.cpu_load.toFixed(2)}`;
        document.getElementById("dash-cpu-bar").style.width = `${Math.max(cpuPercent, 4)}%`;

        document.getElementById("dash-ram-text").textContent = `${ramUsedMb} / ${Math.round(data.ram_total_kb / 1024)} MB`;
        document.getElementById("dash-ram-bar").style.width = `${data.ram_used_percent.toFixed(0)}%`;
        document.getElementById("dash-ram-free").textContent = `Свободно: ${Math.round(data.ram_free_kb / 1024)} MB`;

        // Format uptime
        const s = data.uptime_seconds;
        const hrs = Math.floor(s / 3600);
        const mins = Math.floor((s % 3600) / 60);
        const secs = s % 60;
        document.getElementById("dash-uptime").textContent =
            `${hrs.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;

    } catch (e) {
        console.warn("Status fetch failed", e);
    }
}

async function fetchInterfaces() {
    try {
        const res = await fetch("/api/v1/network/interfaces");
        if (!res.ok) return;
        const ifaces = await res.json();
        const tbody = document.getElementById("ifaces-table");
        if (!tbody) return;

        let totalRx = 0, totalTx = 0;
        tbody.innerHTML = "";

        ifaces.forEach(iface => {
            totalRx += iface.rx_bytes;
            totalTx += iface.tx_bytes;

            const tr = document.createElement("tr");
            tr.innerHTML = `
                <td><strong>${iface.name}</strong></td>
                <td><span class="badge-pill">${iface.if_type}</span></td>
                <td><code>${iface.mac_address}</code></td>
                <td><span class="text-emerald">UP</span></td>
                <td>${formatBytes(iface.rx_bytes)}</td>
                <td>${formatBytes(iface.tx_bytes)}</td>
            `;
            tbody.appendChild(tr);
        });

        // Calculate rate for WAN
        if (prevRx > 0 && prevTx > 0) {
            const rxRate = Math.max(0, (totalRx - prevRx) / 2); // 2 sec poll
            const txRate = Math.max(0, (totalTx - prevTx) / 2);

            rxHistory.shift(); rxHistory.push(rxRate / 1024);
            txHistory.shift(); txHistory.push(txRate / 1024);

            document.getElementById("rx-rate").textContent = `${(rxRate / 1024).toFixed(1)} KB/s`;
            document.getElementById("tx-rate").textContent = `${(txRate / 1024).toFixed(1)} KB/s`;

            drawTrafficChart();
        }
        prevRx = totalRx;
        prevTx = totalTx;

    } catch (e) {
        console.warn("Interfaces fetch failed", e);
    }
}

function formatBytes(bytes) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function startStatsPolling() {
    fetchSystemStatus();
    fetchInterfaces();

    let pollInterval = null;
    function schedulePoll() {
        if (pollInterval) clearInterval(pollInterval);
        if (document.hidden) return; // Completely STOP polling when tab is hidden (Zero CPU load)
        pollInterval = setInterval(() => {
            if (!document.hidden) {
                fetchSystemStatus();
                fetchInterfaces();
            }
        }, 4000); // 4000ms gentle interval saves 50% CPU when active
    }

    document.addEventListener("visibilitychange", () => {
        if (!document.hidden) {
            fetchSystemStatus();
            fetchInterfaces();
            schedulePoll();
        } else {
            if (pollInterval) {
                clearInterval(pollInterval);
                pollInterval = null;
            }
        }
    });

    schedulePoll();
}

function initDashboard() {
    drawTrafficChart();
    window.addEventListener("resize", drawTrafficChart);
}

// VLESS & Hysteria 2 Proxy Management
let currentParsedConfig = null;

function initProxyManager() {
    const parseBtn = document.getElementById("btn-parse-proxy");
    const uriInput = document.getElementById("proxy-uri-input");
    const applyBtn = document.getElementById("btn-apply-proxy");
    const testPingBtn = document.getElementById("btn-test-ping");
    const parsedCard = document.getElementById("proxy-parsed-card");
    const pingResult = document.getElementById("ping-result");
    const customRulesBox = document.getElementById("custom-rules-box");
    const proxyMasterEnable = document.getElementById("proxy-master-enable");
    const proxyMasterLabel = document.getElementById("proxy-master-label");

    function renderParsedProxy(cfg) {
        if (!cfg || !parsedCard) return;
        currentParsedConfig = cfg;
        parsedCard.classList.remove("d-none");

        document.getElementById("parsed-tag").textContent = cfg.tag || "Новый узел";
        document.getElementById("parsed-type-badge").textContent = (cfg.proxy_type || "proxy").toUpperCase();
        document.getElementById("parsed-host").textContent = cfg.server || "-";
        document.getElementById("parsed-port").textContent = cfg.server_port || "-";
        document.getElementById("parsed-security").textContent = cfg.security || "none";
        document.getElementById("parsed-sni").textContent = cfg.sni || "-";
        document.getElementById("parsed-flow").textContent = cfg.flow || "-";
        document.getElementById("parsed-fp").textContent = cfg.fingerprint || "-";
    }

    async function syncProxyStatus() {
        try {
            const res = await fetch("/api/v1/proxy/status");
            const data = await res.json();
            if (!data) return;

            const statusText = document.getElementById("proxy-status-text");
            const proxyPill = document.getElementById("proxy-pill");
            const modeBadge = document.getElementById("proxy-active-mode-badge");

            if (modeBadge) {
                modeBadge.textContent = data.mode === "smart_ru" ? "Smart RU" : data.mode.toUpperCase();
            }

            if (proxyMasterEnable) {
                proxyMasterEnable.checked = !!data.enabled;
                if (proxyMasterLabel) proxyMasterLabel.textContent = data.enabled ? "Включен" : "Отключен";
            }

            if (data.active && data.running) {
                if (statusText) statusText.textContent = "Активен (Подключено)";
                if (proxyPill) {
                    proxyPill.textContent = "Active";
                    proxyPill.className = "badge-status status-good";
                }
            } else if (data.enabled && !data.core_installed) {
                if (statusText) statusText.textContent = "Ядро Sing-Box не установлено в Flash (лимит 8 МБ).";
                if (proxyPill) {
                    proxyPill.textContent = "Core Missing";
                    proxyPill.className = "badge-status status-warning";
                }
            } else if (data.enabled && !data.running) {
                if (statusText) statusText.textContent = "Служба Sing-Box остановлена";
                if (proxyPill) {
                    proxyPill.textContent = "Stopped";
                    proxyPill.className = "badge-status status-warning";
                }
            } else {
                if (statusText) statusText.textContent = "Отключен";
                if (proxyPill) {
                    proxyPill.textContent = "Inactive";
                    proxyPill.className = "badge-status status-neutral";
                }
            }

            // Restore saved URI, parsed config, and mode upon refresh
            if (data.raw_uri && uriInput && !uriInput.value) {
                uriInput.value = data.raw_uri;
            }
            if (data.current_proxy) {
                renderParsedProxy(data.current_proxy);
            }
            if (data.mode) {
                document.querySelectorAll(".radio-card").forEach(c => {
                    const isMode = c.getAttribute("data-mode") === data.mode;
                    c.classList.toggle("active", isMode);
                });
                if (customRulesBox) {
                    customRulesBox.classList.toggle("d-none", data.mode !== "custom");
                }
            }
            if (data.custom_domains && data.custom_domains.length > 0) {
                const domInput = document.getElementById("custom-domains-input");
                if (domInput && !domInput.value) {
                    domInput.value = data.custom_domains.join("\n");
                }
            }
        } catch (err) {
            console.warn("Proxy status sync error:", err);
        }
    }

    syncProxyStatus();

    proxyMasterEnable?.addEventListener("change", async (e) => {
        const enabled = e.target.checked;
        if (proxyMasterLabel) proxyMasterLabel.textContent = enabled ? "Включен" : "Отключен";
        try {
            await fetch("/api/v1/proxy/toggle", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ enabled })
            });
            await syncProxyStatus();
        } catch (err) {
            console.warn("Proxy toggle error:", err);
        }
    });

    // Mode Radio selector
    document.querySelectorAll(".radio-card").forEach(card => {
        card.addEventListener("click", () => {
            document.querySelectorAll(".radio-card").forEach(c => c.classList.remove("active"));
            card.classList.add("active");
            const mode = card.getAttribute("data-mode");
            if (customRulesBox) {
                if (mode === "custom") customRulesBox.classList.remove("d-none");
                else customRulesBox.classList.add("d-none");
            }
        });
    });

    // 1-Click Parse Button
    parseBtn?.addEventListener("click", async () => {
        const uri = uriInput.value.trim();
        if (!uri) {
            alert("Пожалуйста, вставьте ссылку vless://, hysteria2://, hy2://, tuic:// или shadowtls://");
            return;
        }

        parseBtn.textContent = "Распознавание...";
        try {
            const res = await fetch("/api/v1/proxy/import", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ uri })
            });
            const data = await res.json();

            if (data.success && data.config) {
                renderParsedProxy(data.config);
                pingResult.textContent = "";
            } else {
                alert("Ошибка распознавания: " + (data.error || "Неверный формат ключа"));
            }
        } catch (e) {
            alert("Ошибка соединения с API: " + e.message);
        } finally {
            parseBtn.textContent = "Распознать ключ";
        }
    });

    // Test Ping Button
    testPingBtn?.addEventListener("click", async () => {
        if (!currentParsedConfig) return;
        pingResult.textContent = "Проверка...";
        try {
            const res = await fetch("/api/v1/proxy/test", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    host: currentParsedConfig.server,
                    port: currentParsedConfig.server_port
                })
            });
            const data = await res.json();
            if (data.success && data.latency_ms !== null) {
                pingResult.textContent = `Задержка: ${data.latency_ms} ms (Доступен)`;
                pingResult.className = "text-xs text-emerald align-self-center";
            } else {
                pingResult.textContent = `Недоступен (${data.error || "Превышен таймаут"})`;
                pingResult.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            pingResult.textContent = "Ошибка теста: " + e.message;
        }
    });

    // Apply Proxy Configuration
    applyBtn?.addEventListener("click", async () => {
        const uri = uriInput.value.trim();
        if (!uri) {
            alert("Сначала вставьте ключ прокси");
            return;
        }

        const activeRadio = document.querySelector(".radio-card.active");
        const mode = activeRadio ? activeRadio.getAttribute("data-mode") : "smart_ru";
        const customDomains = document.getElementById("custom-domains-input")?.value
            .split(/[\s,]+/).filter(Boolean) || [];

        applyBtn.textContent = "Применение...";
        const statusSpan = document.getElementById("proxy-action-status");
        setTimedStatus(statusSpan, "Применение маршрутов и запуск туннеля...", "info", 0);

        try {
            const res = await fetch("/api/v1/proxy/apply", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    uri,
                    mode,
                    custom_domains: customDomains,
                    custom_ips: [],
                    enabled: proxyMasterEnable ? proxyMasterEnable.checked : true
                })
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(statusSpan, "Конфигурация туннеля сохранена", "success", 3500);
            } else {
                setTimedStatus(statusSpan, data.message || "Ошибка применения туннеля", "error", 4000);
            }
            await syncProxyStatus();
        } catch (e) {
            setTimedStatus(statusSpan, "Ошибка: " + e.message, "error", 4000);
        } finally {
            applyBtn.textContent = "Применить конфигурацию туннеля";
        }
    });

    // ----------------------------------------------------
    // BGP Dynamic Routing for Antizapret
    // ----------------------------------------------------
    const bgpEnable = document.getElementById("bgp-enable");
    const bgpPeerIp = document.getElementById("bgp-peer-ip");
    const bgpPeerAs = document.getElementById("bgp-peer-as");
    const bgpLocalAs = document.getElementById("bgp-local-as");
    const bgpTableName = document.getElementById("bgp-table-name");
    const bgpStatusBadge = document.getElementById("bgp-status-badge");
    const bgpInfoDisplay = document.getElementById("bgp-info-display");
    const btnSaveBgp = document.getElementById("btn-save-bgp");
    const bgpSaveStatus = document.getElementById("bgp-save-status");

    fetch("/api/v1/bgp/status")
        .then(r => r.json())
        .then(data => {
            if (!data) return;
            if (bgpEnable) bgpEnable.checked = !!data.enabled;
            if (bgpPeerIp && data.peer) {
                const parts = data.peer.split(":");
                bgpPeerIp.value = parts[0] || "10.224.0.1";
                if (bgpPeerAs && parts[1]) bgpPeerAs.value = parseInt(parts[1], 10) || 65432;
            }

            if (bgpStatusBadge) {
                if (data.connected) {
                    bgpStatusBadge.textContent = "BGP Подключен";
                    bgpStatusBadge.style.background = "#065f46";
                    bgpStatusBadge.style.color = "#34d399";
                } else if (data.enabled) {
                    bgpStatusBadge.textContent = "Подключение...";
                    bgpStatusBadge.style.background = "#78350f";
                    bgpStatusBadge.style.color = "#fbbf24";
                } else {
                    bgpStatusBadge.textContent = "Отключен";
                    bgpStatusBadge.style.background = "#1f293d";
                    bgpStatusBadge.style.color = "#94a3b8";
                }
            }
            if (bgpInfoDisplay) {
                bgpInfoDisplay.textContent = `Маршрутов: ${data.routes_active || 0} | ${data.status_text || ""}`;
            }
        })
        .catch(e => console.warn("BGP status load failed", e));

    btnSaveBgp?.addEventListener("click", async () => {
        const payload = {
            enabled: bgpEnable ? bgpEnable.checked : false,
            peer_ip: bgpPeerIp ? bgpPeerIp.value.trim() : "10.224.0.1",
            peer_as: bgpPeerAs ? (parseInt(bgpPeerAs.value, 10) || 65432) : 65432,
            auto_sync_interval_hours: 6
        };
        btnSaveBgp.textContent = "Применение...";
        try {
            const res = await fetch("/api/v1/bgp/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(bgpSaveStatus, "Параметры BGP сохранены и применены", "success", 3500);
                if (bgpStatusBadge) {
                    bgpStatusBadge.textContent = payload.enabled ? "BGP Подключен" : "Отключен";
                    bgpStatusBadge.style.background = payload.enabled ? "#065f46" : "#1f293d";
                    bgpStatusBadge.style.color = payload.enabled ? "#34d399" : "#94a3b8";
                }
            } else {
                setTimedStatus(bgpSaveStatus, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(bgpSaveStatus, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveBgp.textContent = "Сохранить параметры BGP";
        }
    });
}

function initZapretManager() {
    const masterEnable = document.getElementById("zapret-master-enable");
    const masterLabel = document.getElementById("zapret-master-label");
    const strategySelect = document.getElementById("zapret-strategy");
    const strategyDesc = document.getElementById("zapret-strategy-desc");
    const customBox = document.getElementById("zapret-custom-box");
    const customArgs = document.getElementById("zapret-custom-args");
    const ytCheckbox = document.getElementById("zapret-youtube");
    const dcCheckbox = document.getElementById("zapret-discord");
    const genCheckbox = document.getElementById("zapret-general");
    const saveBtn = document.getElementById("btn-save-zapret");
    const saveStatus = document.getElementById("zapret-save-status");
    const probeBtn = document.getElementById("btn-zapret-probe");
    const probeResults = document.getElementById("zapret-probe-results");
    const probeRec = document.getElementById("zapret-probe-rec");
    const probeList = document.getElementById("zapret-probe-list");

    const descriptions = {
        "1": "General Multi-Split (split2 + badseq + cutoff=d4) — базовая нарезка TCP пакетов для обхода фильтрации без поддельных заголовков.",
        "2": "Fake TLS SNI (Russian TSPU Evasion v1) — подделка ClientHello с разрывом пакета на 2 части. Идеально для YouTube 4K на большинстве провайдеров РФ.",
        "3": "Disorder + Badsum (Ростелеком, Мегафон, Дом.ру) — отправка фейкового пакета с некорректной чексуммой (ТСПУ перехватывает, сервер назначения отбрасывает).",
        "4": "Discord WebRTC Voice + YouTube 4K (multisplit + any-proto) — многократная фрагментация + поддержка UDP протоколов голосовой связи Discord.",
        "5": "Auto-TTL Desync (autottl=2, badseq, syndata) — автоматический подбор времени жизни пакета для обхода аппаратных фильтров ТСПУ.",
        "6": "Пользовательские параметры (Custom NFQWS Args) — тонкая ручная настройка параметров десинхронизации."
    };

    function updateStrategyUI(val) {
        if (strategyDesc && descriptions[val]) {
            strategyDesc.textContent = descriptions[val];
        }
        if (customBox) {
            if (val === "6") customBox.classList.remove("d-none");
            else customBox.classList.add("d-none");
        }
    }

    strategySelect?.addEventListener("change", (e) => {
        updateStrategyUI(e.target.value);
    });

    masterEnable?.addEventListener("change", (e) => {
        if (masterLabel) masterLabel.textContent = e.target.checked ? "Включен" : "Отключен";
    });

    // Fetch initial config
    fetch("/api/v1/zapret/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (masterEnable) {
                masterEnable.checked = !!cfg.enabled;
                if (masterLabel) masterLabel.textContent = cfg.enabled ? "Включен" : "Отключен";
            }
            if (strategySelect) {
                strategySelect.value = String(cfg.strategy || 2);
                updateStrategyUI(strategySelect.value);
            }
            if (customArgs && cfg.custom_nfqws_args) {
                customArgs.value = cfg.custom_nfqws_args;
            }
            if (ytCheckbox) ytCheckbox.checked = !!cfg.youtube_fix;
            if (dcCheckbox) dcCheckbox.checked = !!cfg.discord_fix;
            if (genCheckbox) genCheckbox.checked = !!cfg.general_dpi;
        })
        .catch(e => console.warn("Zapret config load failed:", e));

    saveBtn?.addEventListener("click", async () => {
        saveBtn.textContent = "Сохранение...";
        const payload = {
            enabled: masterEnable ? masterEnable.checked : true,
            strategy: parseInt(strategySelect?.value || "2", 10),
            youtube_fix: ytCheckbox ? ytCheckbox.checked : true,
            discord_fix: dcCheckbox ? dcCheckbox.checked : true,
            general_dpi: genCheckbox ? genCheckbox.checked : true,
            custom_nfqws_args: customArgs?.value.trim() ? customArgs.value.trim() : null,
        };

        try {
            const res = await fetch("/api/v1/zapret/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (saveStatus) {
                setTimedStatus(saveStatus, data.message || (data.success ? "Параметры Zapret сохранены" : "Ошибка сохранения"), data.success ? "success" : "error", 3500);
            }
        } catch (e) {
            if (saveStatus) {
                setTimedStatus(saveStatus, "Ошибка: " + e.message, "error", 4000);
            }
        } finally {
            saveBtn.textContent = "Сохранить параметры Zapret";
        }
    });

    // Auto-probe
    probeBtn?.addEventListener("click", async () => {
        probeBtn.textContent = "Тестирование...";
        probeBtn.disabled = true;
        try {
            const res = await fetch("/api/v1/zapret/probe", {
                method: "POST",
                headers: { "Content-Type": "application/json" }
            });
            const data = await res.json();
            if (data && data.results) {
                if (probeResults) probeResults.classList.remove("d-none");
                if (probeRec) {
                    probeRec.textContent = `Рекомендована: Стратегия ${data.recommended_strategy}`;
                }
                if (probeList) {
                    probeList.innerHTML = "";
                    data.results.forEach(item => {
                        const div = document.createElement("div");
                        div.className = "p-2 rounded border";
                        div.style.cssText = "background:#090d16; border-color:#1e293b; display:flex; justify-content:space-between; align-items:center;";
                        const isGood = item.status === "success";
                        div.innerHTML = `
                            <div>
                                <span style="font-weight:600; color:${isGood ? '#38bdf8' : '#94a3b8'};">Стратегия ${item.strategy_id}</span>
                                <div class="muted" style="font-size:11px;">${item.strategy_name}</div>
                            </div>
                            <div style="text-align:right;">
                                <span class="badge-pill" style="background:${isGood ? '#065f46' : '#7f1d1d'}; color:${isGood ? '#34d399' : '#f87171'};">
                                    ${isGood ? `${item.latency_ms} ms` : 'Блокировка'}
                                </span>
                            </div>
                        `;
                        probeList.appendChild(div);
                    });
                }
                if (strategySelect && data.recommended_strategy) {
                    strategySelect.value = String(data.recommended_strategy);
                    updateStrategyUI(String(data.recommended_strategy));
                }
            }
        } catch (e) {
            alert("Ошибка при тестировании стратегий: " + e.message);
        } finally {
            probeBtn.textContent = "Запустить тест стратегий";
            probeBtn.disabled = false;
        }
    });
}


function initWifiManager() {
    const saveBtn = document.getElementById("btn-save-wifi");
    const statusSpan = document.getElementById("wifi-save-status");
    const guestEnable = document.getElementById("guest-enable");
    const rssiSlider = document.getElementById("wifi-rssi");
    const rssiVal = document.getElementById("wifi-rssi-val");

    let currentWifiAclList = [];

    function renderWifiAclTags() {
        const container = document.getElementById("wifi-mac-tags");
        const emptyMsg = document.getElementById("wifi-mac-empty-msg");
        if (!container) return;
        container.innerHTML = "";
        if (currentWifiAclList.length === 0) {
            if (emptyMsg) emptyMsg.classList.remove("d-none");
        } else {
            if (emptyMsg) emptyMsg.classList.add("d-none");
            currentWifiAclList.forEach((mac, idx) => {
                const tag = document.createElement("span");
                tag.className = "badge-pill";
                tag.style.cssText = "display:inline-flex; align-items:center; gap:6px; padding:4px 8px; background:#161f30; border:1px solid #2d3b55; font-family:monospace;";
                
                const textSpan = document.createElement("span");
                textSpan.textContent = mac;
                tag.appendChild(textSpan);

                const delBtn = document.createElement("button");
                delBtn.type = "button";
                delBtn.className = "btn btn-xs btn-outline-danger";
                delBtn.textContent = "Удалить";
                delBtn.title = "Удалить MAC";
                delBtn.addEventListener("click", () => {
                    currentWifiAclList.splice(idx, 1);
                    renderWifiAclTags();
                });
                tag.appendChild(delBtn);

                container.appendChild(tag);
            });
        }
    }

    const btnAddMac = document.getElementById("btn-add-wifi-mac");
    const inputMac = document.getElementById("wifi-mac-input");
    btnAddMac?.addEventListener("click", () => {
        if (!inputMac) return;
        const raw = inputMac.value.trim().toUpperCase();
        const macRegex = /^([0-9A-F]{2}[:-]){5}([0-9A-F]{2})$/;
        if (!macRegex.test(raw)) {
            alert("Некорректный формат MAC-адреса! Пример: AA:BB:CC:DD:EE:FF");
            return;
        }
        const normalized = raw.replace(/-/g, ":");
        if (!currentWifiAclList.includes(normalized)) {
            currentWifiAclList.push(normalized);
            renderWifiAclTags();
        }
        inputMac.value = "";
    });

    if (rssiSlider && rssiVal) {
        rssiSlider.addEventListener("input", (e) => {
            rssiVal.textContent = e.target.value + " dBm";
        });
    }

    // Load initial settings
    fetch("/api/v1/wifi/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            // 2.4G
            if (document.getElementById("wifi2-ssid")) document.getElementById("wifi2-ssid").value = cfg.wifi_2g_ssid || "";
            if (document.getElementById("wifi2-pass")) document.getElementById("wifi2-pass").value = cfg.wifi_2g_password || "";
            if (document.getElementById("wifi2-sec")) document.getElementById("wifi2-sec").value = cfg.wifi_2g_security || "WPA2PSK";
            if (document.getElementById("wifi2-mode")) document.getElementById("wifi2-mode").value = cfg.wifi_2g_mode || "11bgn";
            if (document.getElementById("wifi2-chan")) document.getElementById("wifi2-chan").value = cfg.wifi_2g_channel || 0;
            if (document.getElementById("wifi2-bw")) document.getElementById("wifi2-bw").value = cfg.wifi_2g_bandwidth !== undefined ? cfg.wifi_2g_bandwidth : 0;
            if (document.getElementById("wifi2-pwr")) document.getElementById("wifi2-pwr").value = cfg.wifi_2g_tx_power || 100;
            if (document.getElementById("wifi2-pmf")) document.getElementById("wifi2-pmf").value = cfg.wifi_2g_pmf !== undefined ? cfg.wifi_2g_pmf : 1;
            if (document.getElementById("wifi2-sgi")) document.getElementById("wifi2-sgi").checked = !!cfg.wifi_2g_short_gi;
            if (document.getElementById("wifi2-iso")) document.getElementById("wifi2-iso").checked = !!cfg.wifi_2g_isolate;
            if (document.getElementById("wifi2-hide")) document.getElementById("wifi2-hide").checked = !!cfg.wifi_2g_hidden;

            // 5G
            if (document.getElementById("wifi5-ssid")) document.getElementById("wifi5-ssid").value = cfg.wifi_5g_ssid || "";
            if (document.getElementById("wifi5-pass")) document.getElementById("wifi5-pass").value = cfg.wifi_5g_password || "";
            if (document.getElementById("wifi5-sec")) document.getElementById("wifi5-sec").value = cfg.wifi_5g_security || "WPA2PSK";
            if (document.getElementById("wifi5-mode")) document.getElementById("wifi5-mode").value = cfg.wifi_5g_mode || "11vht_ac";
            if (document.getElementById("wifi5-chan")) document.getElementById("wifi5-chan").value = cfg.wifi_5g_channel || 36;
            if (document.getElementById("wifi5-bw")) document.getElementById("wifi5-bw").value = cfg.wifi_5g_bandwidth !== undefined ? cfg.wifi_5g_bandwidth : 0;
            if (document.getElementById("wifi5-pwr")) document.getElementById("wifi5-pwr").value = cfg.wifi_5g_tx_power || 100;
            if (document.getElementById("wifi5-pmf")) document.getElementById("wifi5-pmf").value = cfg.wifi_5g_pmf !== undefined ? cfg.wifi_5g_pmf : 1;
            if (document.getElementById("wifi5-sgi")) document.getElementById("wifi5-sgi").checked = !!cfg.wifi_5g_short_gi;
            if (document.getElementById("wifi5-bf")) document.getElementById("wifi5-bf").checked = !!cfg.wifi_5g_beamforming;
            if (document.getElementById("wifi5-stbc")) document.getElementById("wifi5-stbc").checked = !!cfg.wifi_5g_stbc_ldpc;
            if (document.getElementById("wifi5-iso")) document.getElementById("wifi5-iso").checked = !!cfg.wifi_5g_isolate;
            if (document.getElementById("wifi5-hide")) document.getElementById("wifi5-hide").checked = !!cfg.wifi_5g_hidden;

            // Roaming & Acceleration
            if (document.getElementById("wifi-roaming")) document.getElementById("wifi-roaming").checked = !!cfg.fast_roaming_80211k;
            if (document.getElementById("wifi-ft")) document.getElementById("wifi-ft").checked = !!cfg.fast_roaming_80211r;
            if (document.getElementById("wifi-band-steering")) document.getElementById("wifi-band-steering").checked = !!cfg.band_steering;
            if (document.getElementById("wifi-burst")) document.getElementById("wifi-burst").checked = !!cfg.tx_burst;
            if (document.getElementById("wifi-wmm")) document.getElementById("wifi-wmm").checked = !!cfg.wmm_qos;
            if (rssiSlider && cfg.rssi_roam_threshold) {
                rssiSlider.value = cfg.rssi_roam_threshold;
                rssiVal.textContent = cfg.rssi_roam_threshold + " dBm";
            }

            // Guest
            if (guestEnable) guestEnable.checked = !!cfg.guest_enabled;
            if (document.getElementById("guest-ssid")) document.getElementById("guest-ssid").value = cfg.guest_ssid || "";
            if (document.getElementById("guest-pass")) document.getElementById("guest-pass").value = cfg.guest_password || "";
            if (document.getElementById("guest-bw-limit")) document.getElementById("guest-bw-limit").value = cfg.guest_bandwidth_limit_mbps || 20;
            if (document.getElementById("guest-iso-lan")) document.getElementById("guest-iso-lan").checked = !!cfg.guest_isolate_lan;

            // Professional RF & ACL
            if (document.getElementById("wifi-country")) document.getElementById("wifi-country").value = cfg.country_code || "RU";
            if (document.getElementById("wifi-preamble")) document.getElementById("wifi-preamble").value = cfg.preamble_short ? "short" : "long";
            if (document.getElementById("wifi-turbo-qam")) document.getElementById("wifi-turbo-qam").checked = cfg.turbo_qam !== undefined ? !!cfg.turbo_qam : true;
            if (document.getElementById("wifi-green-ap")) document.getElementById("wifi-green-ap").checked = !!cfg.green_ap;
            if (document.getElementById("wifi-airtime-fairness")) document.getElementById("wifi-airtime-fairness").checked = cfg.airtime_fairness !== undefined ? !!cfg.airtime_fairness : true;
            if (document.getElementById("wifi-mac-mode")) document.getElementById("wifi-mac-mode").value = cfg.mac_filter_mode !== undefined ? cfg.mac_filter_mode : 0;
            currentWifiAclList = Array.isArray(cfg.mac_filter_list) ? [...cfg.mac_filter_list] : [];
            renderWifiAclTags();

            // Scheduler
            if (document.getElementById("wifi-sched-enable")) document.getElementById("wifi-sched-enable").checked = !!cfg.wifi_schedule_enabled;
            if (document.getElementById("wifi-sched-start")) document.getElementById("wifi-sched-start").value = cfg.wifi_schedule_start || "01:00";
            if (document.getElementById("wifi-sched-end")) document.getElementById("wifi-sched-end").value = cfg.wifi_schedule_end || "07:00";
        })
        .catch(e => console.warn("Wi-Fi config load failed", e));

    saveBtn?.addEventListener("click", async () => {
        const payload = {
            wifi_2g_enabled: true,
            wifi_2g_ssid: document.getElementById("wifi2-ssid").value,
            wifi_2g_password: document.getElementById("wifi2-pass").value,
            wifi_2g_security: document.getElementById("wifi2-sec").value,
            wifi_2g_mode: document.getElementById("wifi2-mode").value,
            wifi_2g_channel: parseInt(document.getElementById("wifi2-chan").value, 10),
            wifi_2g_bandwidth: parseInt(document.getElementById("wifi2-bw").value, 10),
            wifi_2g_tx_power: parseInt(document.getElementById("wifi2-pwr").value, 10),
            wifi_2g_pmf: parseInt(document.getElementById("wifi2-pmf").value, 10),
            wifi_2g_short_gi: document.getElementById("wifi2-sgi").checked,
            wifi_2g_isolate: document.getElementById("wifi2-iso").checked,
            wifi_2g_hidden: document.getElementById("wifi2-hide").checked,
            wifi_2g_dtim: 1,
            wifi_2g_beacon: 100,

            wifi_5g_enabled: true,
            wifi_5g_ssid: document.getElementById("wifi5-ssid").value,
            wifi_5g_password: document.getElementById("wifi5-pass").value,
            wifi_5g_security: document.getElementById("wifi5-sec").value,
            wifi_5g_mode: document.getElementById("wifi5-mode").value,
            wifi_5g_channel: parseInt(document.getElementById("wifi5-chan").value, 10),
            wifi_5g_bandwidth: parseInt(document.getElementById("wifi5-bw").value, 10),
            wifi_5g_tx_power: parseInt(document.getElementById("wifi5-pwr").value, 10),
            wifi_5g_pmf: parseInt(document.getElementById("wifi5-pmf").value, 10),
            wifi_5g_short_gi: document.getElementById("wifi5-sgi").checked,
            wifi_5g_beamforming: document.getElementById("wifi5-bf").checked,
            wifi_5g_stbc_ldpc: document.getElementById("wifi5-stbc").checked,
            wifi_5g_isolate: document.getElementById("wifi5-iso").checked,
            wifi_5g_hidden: document.getElementById("wifi5-hide").checked,

            band_steering: document.getElementById("wifi-band-steering").checked,
            fast_roaming_80211k: document.getElementById("wifi-roaming").checked,
            fast_roaming_80211v: document.getElementById("wifi-roaming").checked,
            fast_roaming_80211r: document.getElementById("wifi-ft").checked,
            rssi_roam_threshold: parseInt(document.getElementById("wifi-rssi").value, 10),
            tx_burst: document.getElementById("wifi-burst").checked,
            packet_aggregate: true,
            wmm_qos: document.getElementById("wifi-wmm").checked,

            guest_enabled: guestEnable.checked,
            guest_ssid: document.getElementById("guest-ssid").value,
            guest_password: document.getElementById("guest-pass").value,
            guest_isolate_lan: document.getElementById("guest-iso-lan").checked,
            guest_bandwidth_limit_mbps: parseInt(document.getElementById("guest-bw-limit").value, 10) || 20,

            preamble_short: document.getElementById("wifi-preamble") ? document.getElementById("wifi-preamble").value === "short" : true,
            turbo_qam: document.getElementById("wifi-turbo-qam") ? document.getElementById("wifi-turbo-qam").checked : true,
            green_ap: document.getElementById("wifi-green-ap") ? document.getElementById("wifi-green-ap").checked : false,
            airtime_fairness: document.getElementById("wifi-airtime-fairness") ? document.getElementById("wifi-airtime-fairness").checked : true,
            country_code: document.getElementById("wifi-country") ? document.getElementById("wifi-country").value : "RU",
            mac_filter_mode: parseInt(document.getElementById("wifi-mac-mode")?.value, 10) || 0,
            mac_filter_list: currentWifiAclList,
            wifi_schedule_enabled: document.getElementById("wifi-sched-enable") ? document.getElementById("wifi-sched-enable").checked : false,
            wifi_schedule_start: document.getElementById("wifi-sched-start")?.value || "01:00",
            wifi_schedule_end: document.getElementById("wifi-sched-end")?.value || "07:00",
        };

        saveBtn.textContent = "Применение к радиомодулям...";
        setTimedStatus(statusSpan, "Запись параметров во Flash-память...", "info", 0);

        try {
            const res = await fetch("/api/v1/wifi/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(statusSpan, "Настройки Wi-Fi сохранены во Flash-память", "success", 3500);
            } else {
                setTimedStatus(statusSpan, data.message || "Ошибка запуска туннеля", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(statusSpan, "Ошибка сохранения: " + e.message, "error", 4000);
        } finally {
            saveBtn.textContent = "Сохранить параметры Wi-Fi";
        }
    });
}

// Network, DNS, WAN, LAN DHCP & Port Forwarding Manager
function initNetworkManager() {
    // ----------------------------------------------------
    // 1. DNS & DoH / DoT Manager
    // ----------------------------------------------------
    const dnsProviderSelect = document.getElementById("dns-provider");
    const dnsPrimaryInput = document.getElementById("dns-primary");
    const dnsSecondaryInput = document.getElementById("dns-secondary");
    const dnsDohUrlInput = document.getElementById("dns-doh-url");
    const dnsDohEnable = document.getElementById("dns-doh-enable");
    const dnsForceRouter = document.getElementById("dns-force-router");
    const dnsAntiRebind = document.getElementById("dns-anti-rebind");
    const dnsAdblockEnable = document.getElementById("dns-adblock-enable");
    const dnsCustomHosts = document.getElementById("dns-custom-hosts");
    const btnSaveDns = document.getElementById("btn-save-dns");
    const dnsStatusSpan = document.getElementById("dns-save-status");

    const dnsPresets = {
        cloudflare: { p: "1.1.1.1", s: "1.0.0.1", doh: "https://cloudflare-dns.com/dns-query" },
        google: { p: "8.8.8.8", s: "8.8.4.4", doh: "https://dns.google/dns-query" },
        adguard: { p: "94.140.14.14", s: "94.140.15.15", doh: "https://dns.adguard-dns.com/dns-query" },
        quad9: { p: "9.9.9.9", s: "149.112.112.112", doh: "https://dns.quad9.net/dns-query" },
        yandex: { p: "77.88.8.8", s: "77.88.8.1", doh: "" },
    };

    dnsProviderSelect?.addEventListener("change", (e) => {
        const val = e.target.value;
        if (dnsPresets[val]) {
            dnsPrimaryInput.value = dnsPresets[val].p;
            dnsSecondaryInput.value = dnsPresets[val].s;
            dnsDohUrlInput.value = dnsPresets[val].doh;
        }
    });

    // Load initial DNS config
    fetch("/api/v1/dns/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (dnsProviderSelect) dnsProviderSelect.value = cfg.provider || "cloudflare";
            if (dnsPrimaryInput) dnsPrimaryInput.value = cfg.primary_dns || "1.1.1.1";
            if (dnsSecondaryInput) dnsSecondaryInput.value = cfg.secondary_dns || "1.0.0.1";
            if (dnsDohEnable) dnsDohEnable.checked = !!cfg.doh_enabled;
            if (dnsDohUrlInput) dnsDohUrlInput.value = cfg.doh_url || "";
            if (dnsForceRouter) dnsForceRouter.checked = !!cfg.force_router_dns;
            if (dnsAntiRebind) dnsAntiRebind.checked = !!cfg.anti_dns_rebind;
            if (dnsAdblockEnable) dnsAdblockEnable.checked = !!cfg.adblock_enabled;
            if (dnsCustomHosts && cfg.custom_hosts) {
                dnsCustomHosts.value = cfg.custom_hosts.join("\n");
            }
        })
        .catch(e => console.warn("DNS config load failed", e));

    btnSaveDns?.addEventListener("click", async () => {
        const hosts = (dnsCustomHosts?.value || "")
            .split("\n")
            .map(s => s.trim())
            .filter(Boolean);

        const payload = {
            provider: dnsProviderSelect.value,
            primary_dns: dnsPrimaryInput.value.trim(),
            secondary_dns: dnsSecondaryInput.value.trim(),
            doh_enabled: dnsDohEnable.checked,
            doh_url: dnsDohUrlInput.value.trim(),
            force_router_dns: dnsForceRouter.checked,
            anti_dns_rebind: dnsAntiRebind.checked,
            adblock_enabled: dnsAdblockEnable.checked,
            custom_hosts: hosts,
        };

        btnSaveDns.textContent = "Сохранение...";
        try {
            const res = await fetch("/api/v1/dns/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(dnsStatusSpan, "Настройки DNS сохранены и применены", "success", 3500);
            } else {
                setTimedStatus(dnsStatusSpan, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(dnsStatusSpan, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveDns.textContent = "Сохранить параметры DNS";
        }
    });

    // ----------------------------------------------------
    // 2. WAN & Internet Connection Manager
    // ----------------------------------------------------
    const wanTypeSelect = document.getElementById("wan-type");
    const wanStaticBox = document.getElementById("wan-static-box");
    const wanPppoeBox = document.getElementById("wan-pppoe-box");
    const btnSaveWan = document.getElementById("btn-save-wan");
    const wanStatusSpan = document.getElementById("wan-save-status");

    function updateWanVisibility() {
        const val = wanTypeSelect.value;
        if (val === "static") {
            wanStaticBox?.classList.remove("d-none");
            wanPppoeBox?.classList.add("d-none");
        } else if (val === "pppoe") {
            wanStaticBox?.classList.add("d-none");
            wanPppoeBox?.classList.remove("d-none");
        } else {
            wanStaticBox?.classList.add("d-none");
            wanPppoeBox?.classList.add("d-none");
        }
    }
    wanTypeSelect?.addEventListener("change", updateWanVisibility);

    let currentWanConfig = {};

    fetch("/api/v1/wan/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            currentWanConfig = cfg;
            if (wanTypeSelect) {
                wanTypeSelect.value = cfg.connection_type || "dhcp";
                updateWanVisibility();
            }
            if (document.getElementById("wan-mtu")) document.getElementById("wan-mtu").value = cfg.mtu || 1500;
            if (document.getElementById("wan-mac")) document.getElementById("wan-mac").value = cfg.clone_mac || "";
            if (document.getElementById("wan-static-ip")) document.getElementById("wan-static-ip").value = cfg.static_ip || "";
            if (document.getElementById("wan-static-mask")) document.getElementById("wan-static-mask").value = cfg.static_netmask || "255.255.255.0";
            if (document.getElementById("wan-static-gw")) document.getElementById("wan-static-gw").value = cfg.static_gateway || "";
            if (document.getElementById("wan-pppoe-user")) document.getElementById("wan-pppoe-user").value = cfg.pppoe_user || "";
            if (document.getElementById("wan-pppoe-pass")) document.getElementById("wan-pppoe-pass").value = cfg.pppoe_pass || "";
            if (document.getElementById("wan-pppoe-service")) document.getElementById("wan-pppoe-service").value = cfg.pppoe_service || "";
            if (document.getElementById("wan-vlan-internet")) document.getElementById("wan-vlan-internet").value = cfg.vlan_internet_id || "";
            if (document.getElementById("wan-vlan-iptv")) document.getElementById("wan-vlan-iptv").value = cfg.vlan_iptv_id || "";
            if (document.getElementById("wan-iptv-port")) document.getElementById("wan-iptv-port").value = cfg.iptv_port || 4;
            if (document.getElementById("wan-hw-nat")) document.getElementById("wan-hw-nat").checked = !!cfg.hw_nat;
            if (document.getElementById("wan-upnp")) document.getElementById("wan-upnp").checked = !!cfg.upnp;
            if (document.getElementById("wan-dmz-ip")) document.getElementById("wan-dmz-ip").value = cfg.dmz_ip || "";

            renderPortForwardingRules(cfg.port_forwards || []);
        })
        .catch(e => console.warn("WAN config load failed", e));

    btnSaveWan?.addEventListener("click", async () => {
        const payload = {
            ...currentWanConfig,
            connection_type: wanTypeSelect.value,
            mtu: parseInt(document.getElementById("wan-mtu").value, 10) || 1500,
            clone_mac: document.getElementById("wan-mac").value.trim() || null,
            static_ip: document.getElementById("wan-static-ip").value.trim() || null,
            static_netmask: document.getElementById("wan-static-mask").value.trim() || null,
            static_gateway: document.getElementById("wan-static-gw").value.trim() || null,
            pppoe_user: document.getElementById("wan-pppoe-user").value.trim() || null,
            pppoe_pass: document.getElementById("wan-pppoe-pass").value || null,
            pppoe_service: document.getElementById("wan-pppoe-service").value.trim() || null,
            vlan_internet_id: parseInt(document.getElementById("wan-vlan-internet").value, 10) || null,
            vlan_iptv_id: parseInt(document.getElementById("wan-vlan-iptv").value, 10) || null,
            iptv_port: parseInt(document.getElementById("wan-iptv-port").value, 10) || null,
            hw_nat: document.getElementById("wan-hw-nat").checked,
            upnp: document.getElementById("wan-upnp").checked,
        };

        btnSaveWan.textContent = "Сохранение...";
        try {
            const res = await fetch("/api/v1/wan/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                currentWanConfig = payload;
                setTimedStatus(wanStatusSpan, "Настройки WAN сохранены", "success", 3500);
            } else {
                setTimedStatus(wanStatusSpan, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(wanStatusSpan, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveWan.textContent = "Сохранить параметры WAN";
        }
    });

    // ----------------------------------------------------
    // 3. LAN & DHCP Server Manager
    // ----------------------------------------------------
    const btnSaveLan = document.getElementById("btn-save-lan");
    const lanStatusSpan = document.getElementById("lan-save-status");
    const staticLeasesTbody = document.querySelector("#static-leases-table tbody");
    const btnAddStaticLease = document.getElementById("btn-add-static-lease");

    let currentStaticLeases = [];

    function renderStaticLeasesTable() {
        if (!staticLeasesTbody) return;
        staticLeasesTbody.innerHTML = "";
        if (currentStaticLeases.length === 0) {
            staticLeasesTbody.innerHTML = `<tr><td colspan="4" class="text-center muted">Нет статических привязок IP. Нажмите «+ Добавить привязку».</td></tr>`;
            return;
        }
        currentStaticLeases.forEach((lease, idx) => {
            const tr = document.createElement("tr");
            tr.innerHTML = `
                <td><input type="text" class="form-control text-xs lease-mac" value="${lease.mac || ''}" placeholder="AA:BB:CC:DD:EE:FF"></td>
                <td><input type="text" class="form-control text-xs lease-ip" value="${lease.ip || ''}" placeholder="192.168.1.50"></td>
                <td><input type="text" class="form-control text-xs lease-host" value="${lease.hostname || ''}" placeholder="Smart-TV"></td>
                <td><button class="btn btn-xs btn-outline-danger btn-del-lease" data-idx="${idx}">Удалить</button></td>
            `;
            staticLeasesTbody.appendChild(tr);
        });

        staticLeasesTbody.querySelectorAll(".btn-del-lease").forEach(btn => {
            btn.addEventListener("click", (e) => {
                const idx = parseInt(e.target.dataset.idx, 10);
                currentStaticLeases.splice(idx, 1);
                renderStaticLeasesTable();
            });
        });
    }

    btnAddStaticLease?.addEventListener("click", () => {
        currentStaticLeases.push({ mac: "", ip: "192.168.1.", hostname: "" });
        renderStaticLeasesTable();
    });

    fetch("/api/v1/lan/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (document.getElementById("lan-ip")) document.getElementById("lan-ip").value = cfg.lan_ip || "192.168.1.1";
            if (document.getElementById("lan-mask")) document.getElementById("lan-mask").value = cfg.lan_netmask || "255.255.255.0";
            if (document.getElementById("lan-dhcp-start")) document.getElementById("lan-dhcp-start").value = cfg.dhcp_start || "192.168.1.100";
            if (document.getElementById("lan-dhcp-end")) document.getElementById("lan-dhcp-end").value = cfg.dhcp_end || "192.168.1.244";
            if (document.getElementById("lan-dhcp-enable")) document.getElementById("lan-dhcp-enable").checked = !!cfg.dhcp_enabled;
            if (document.getElementById("lan-lease-hours")) document.getElementById("lan-lease-hours").value = cfg.lease_time_hours || 24;

            currentStaticLeases = cfg.static_leases || [];
            renderStaticLeasesTable();
        })
        .catch(e => console.warn("LAN config load failed", e));

    btnSaveLan?.addEventListener("click", async () => {
        // Collect static leases from table rows
        const rows = staticLeasesTbody?.querySelectorAll("tr") || [];
        const leases = [];
        rows.forEach(tr => {
            const mac = tr.querySelector(".lease-mac")?.value.trim();
            const ip = tr.querySelector(".lease-ip")?.value.trim();
            const host = tr.querySelector(".lease-host")?.value.trim();
            if (mac && ip) {
                leases.push({ mac, ip, hostname: host || "device" });
            }
        });

        const payload = {
            lan_ip: document.getElementById("lan-ip").value.trim(),
            lan_netmask: document.getElementById("lan-mask").value.trim(),
            dhcp_enabled: document.getElementById("lan-dhcp-enable").checked,
            dhcp_start: document.getElementById("lan-dhcp-start").value.trim(),
            dhcp_end: document.getElementById("lan-dhcp-end").value.trim(),
            lease_time_hours: parseInt(document.getElementById("lan-lease-hours").value, 10) || 24,
            domain_name: "lan",
            static_leases: leases,
        };

        btnSaveLan.textContent = "Сохранение...";
        try {
            const res = await fetch("/api/v1/lan/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                currentStaticLeases = leases;
                setTimedStatus(lanStatusSpan, "Настройки LAN и статические привязки сохранены", "success", 3500);
            } else {
                setTimedStatus(lanStatusSpan, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(lanStatusSpan, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveLan.textContent = "Сохранить параметры LAN и DHCP";
        }
    });

    // Active DHCP Leases Monitor
    const activeDhcpTbody = document.querySelector("#active-dhcp-table tbody");
    const btnRefreshDhcp = document.getElementById("btn-refresh-dhcp-leases");

    function refreshDhcpLeases() {
        if (!activeDhcpTbody) return;
        fetch("/api/v1/dhcp/leases")
            .then(r => r.json())
            .then(leases => {
                activeDhcpTbody.innerHTML = "";
                if (!leases || leases.length === 0) {
                    activeDhcpTbody.innerHTML = `<tr><td colspan="4" class="text-center muted">Нет активных клиентов DHCP</td></tr>`;
                    return;
                }
                leases.forEach(client => {
                    const tr = document.createElement("tr");
                    tr.innerHTML = `
                        <td><strong>${client.hostname || 'Unknown'}</strong></td>
                        <td class="text-cyan font-bold">${client.ip}</td>
                        <td class="font-mono text-xs">${client.mac}</td>
                        <td class="muted text-xs">${client.expires}</td>
                    `;
                    activeDhcpTbody.appendChild(tr);
                });
            })
            .catch(e => console.warn("Failed to load active leases", e));
    }
    btnRefreshDhcp?.addEventListener("click", refreshDhcpLeases);
    refreshDhcpLeases();

    // ----------------------------------------------------
    // 4. Port Forwarding Rules Manager
    // ----------------------------------------------------
    const pfTableTbody = document.querySelector("#pf-rules-table tbody");
    const btnAddPfRule = document.getElementById("btn-add-pf-rule");
    const btnSavePf = document.getElementById("btn-save-pf");
    const pfStatusSpan = document.getElementById("pf-save-status");

    let currentPfRules = [];

    function renderPortForwardingRules(rules) {
        currentPfRules = rules;
        if (!pfTableTbody) return;
        pfTableTbody.innerHTML = "";
        if (currentPfRules.length === 0) {
            pfTableTbody.innerHTML = `<tr><td colspan="6" class="text-center muted">Нет правил проброса портов. Нажмите «+ Добавить правило».</td></tr>`;
            return;
        }
        currentPfRules.forEach((rule, idx) => {
            const tr = document.createElement("tr");
            tr.innerHTML = `
                <td><input type="text" class="form-control text-xs pf-name" value="${rule.name || ''}" placeholder="HTTP"></td>
                <td>
                    <select class="form-control text-xs pf-proto">
                        <option value="TCP" ${rule.proto === 'TCP' ? 'selected' : ''}>TCP</option>
                        <option value="UDP" ${rule.proto === 'UDP' ? 'selected' : ''}>UDP</option>
                        <option value="BOTH" ${rule.proto === 'BOTH' ? 'selected' : ''}>TCP+UDP</option>
                    </select>
                </td>
                <td><input type="number" class="form-control text-xs pf-ext-port" value="${rule.ext_port || ''}" placeholder="8080"></td>
                <td><input type="text" class="form-control text-xs pf-int-ip" value="${rule.int_ip || ''}" placeholder="192.168.1.10"></td>
                <td><input type="number" class="form-control text-xs pf-int-port" value="${rule.int_port || ''}" placeholder="80"></td>
                <td><button class="btn btn-xs btn-outline-danger btn-del-pf" data-idx="${idx}">Удалить</button></td>
            `;
            pfTableTbody.appendChild(tr);
        });

        pfTableTbody.querySelectorAll(".btn-del-pf").forEach(btn => {
            btn.addEventListener("click", (e) => {
                const idx = parseInt(e.target.dataset.idx, 10);
                currentPfRules.splice(idx, 1);
                renderPortForwardingRules(currentPfRules);
            });
        });
    }

    btnAddPfRule?.addEventListener("click", () => {
        currentPfRules.push({ name: "Service", proto: "TCP", ext_port: 8080, int_ip: "192.168.1.100", int_port: 80 });
        renderPortForwardingRules(currentPfRules);
    });

    btnSavePf?.addEventListener("click", async () => {
        const rows = pfTableTbody?.querySelectorAll("tr") || [];
        const rules = [];
        rows.forEach(tr => {
            const name = tr.querySelector(".pf-name")?.value.trim();
            const proto = tr.querySelector(".pf-proto")?.value;
            const extPort = parseInt(tr.querySelector(".pf-ext-port")?.value, 10);
            const intIp = tr.querySelector(".pf-int-ip")?.value.trim();
            const intPort = parseInt(tr.querySelector(".pf-int-port")?.value, 10);
            if (extPort && intIp && intPort) {
                rules.push({ name: name || "Rule", proto: proto || "TCP", ext_port: extPort, int_ip: intIp, int_port: intPort });
            }
        });

        const dmzIp = document.getElementById("wan-dmz-ip")?.value.trim() || null;

        const payload = {
            ...currentWanConfig,
            dmz_ip: dmzIp,
            port_forwards: rules,
        };

        btnSavePf.textContent = "Применение...";
        try {
            const res = await fetch("/api/v1/wan/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                currentWanConfig = payload;
                setTimedStatus(pfStatusSpan, "Правила проброса портов применены", "success", 3500);
            } else {
                setTimedStatus(pfStatusSpan, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(pfStatusSpan, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSavePf.textContent = "Применить правила";
        }
    });

    // ----------------------------------------------------
    // 5. Physical Ethernet Switch Ports Monitor
    // ----------------------------------------------------
    const switchPortsGrid = document.getElementById("switch-ports-grid");
    const btnRefreshSwitch = document.getElementById("btn-refresh-switch");

    async function loadSwitchPorts() {
        if (!switchPortsGrid) return;
        try {
            const res = await fetch("/api/v1/switch/ports");
            const ports = await res.json();
            switchPortsGrid.innerHTML = "";
            ports.forEach(p => {
                const statusClass = p.link_up ? "port-up" : "port-down";
                const statusText = p.link_up ? "UP" : "DOWN";
                const speedText = p.link_up ? `${p.speed_mbps} Mbps ${p.duplex}` : "Отключен";
                const div = document.createElement("div");
                div.className = "card";
                div.style.padding = "0.75rem";
                div.style.textAlign = "center";
                div.innerHTML = `
                    <div style="margin-bottom: 0.4rem;"><span class="port-badge ${statusClass}">${statusText}</span></div>
                    <div style="font-weight: 600; font-size: 0.85rem;">${p.port_name}</div>
                    <div class="muted" style="margin-top: 0.25rem; font-size: 0.725rem; font-family: var(--font-mono);">${speedText}</div>
                `;
                switchPortsGrid.appendChild(div);
            });
        } catch (e) {
            console.warn("Switch ports query failed", e);
        }
    }

    btnRefreshSwitch?.addEventListener("click", loadSwitchPorts);
    loadSwitchPorts();

    // ----------------------------------------------------
    // 6. IPv6 Dual-Stack Settings
    // ----------------------------------------------------
    const ipv6Enable = document.getElementById("ipv6-enable");
    const ipv6Type = document.getElementById("ipv6-type");
    const ipv6Dns1 = document.getElementById("ipv6-dns1");
    const ipv6Dns2 = document.getElementById("ipv6-dns2");
    const btnSaveIpv6 = document.getElementById("btn-save-ipv6");
    const ipv6Status = document.getElementById("ipv6-save-status");

    fetch("/api/v1/ipv6/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (ipv6Enable) ipv6Enable.checked = !!cfg.enabled;
            if (ipv6Type && cfg.connection_type) ipv6Type.value = cfg.connection_type;
            if (ipv6Dns1 && cfg.primary_dns) ipv6Dns1.value = cfg.primary_dns;
            if (ipv6Dns2 && cfg.secondary_dns) ipv6Dns2.value = cfg.secondary_dns;
        })
        .catch(e => console.warn("IPv6 load failed", e));

    btnSaveIpv6?.addEventListener("click", async () => {
        const payload = {
            enabled: ipv6Enable.checked,
            connection_type: ipv6Type.value,
            primary_dns: ipv6Dns1.value.trim(),
            secondary_dns: ipv6Dns2.value.trim(),
        };
        btnSaveIpv6.textContent = "Сохранение...";
        try {
            const res = await fetch("/api/v1/ipv6/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(ipv6Status, "Настройки IPv6 сохранены", "success", 3500);
            } else {
                setTimedStatus(ipv6Status, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(ipv6Status, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveIpv6.textContent = "Сохранить параметры IPv6";
        }
    });

    // ----------------------------------------------------
    // 7. IPTV, Multicast & Udpxy Streamer
    // ----------------------------------------------------
    const iptvSnooping = document.getElementById("iptv-snooping");
    const iptvProxy = document.getElementById("iptv-proxy");
    const iptvUdpxy = document.getElementById("iptv-udpxy");
    const iptvPort = document.getElementById("iptv-udpxy-port");
    const btnSaveIptv = document.getElementById("btn-save-iptv");
    const iptvStatus = document.getElementById("iptv-save-status");

    fetch("/api/v1/iptv/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (iptvSnooping) iptvSnooping.checked = !!cfg.igmp_snooping;
            if (iptvProxy) iptvProxy.checked = !!cfg.igmp_proxy_enabled;
            if (iptvUdpxy) iptvUdpxy.checked = !!cfg.udpxy_enabled;
            if (iptvPort && cfg.udpxy_port) iptvPort.value = cfg.udpxy_port;
        })
        .catch(e => console.warn("IPTV load failed", e));

    btnSaveIptv?.addEventListener("click", async () => {
        const payload = {
            igmp_snooping: iptvSnooping.checked,
            igmp_proxy_enabled: iptvProxy.checked,
            udpxy_enabled: iptvUdpxy.checked,
            udpxy_port: parseInt(iptvPort.value, 10) || 4028,
        };
        btnSaveIptv.textContent = "Применение...";
        try {
            const res = await fetch("/api/v1/iptv/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(iptvStatus, "Настройки IPTV и Udpxy применены", "success", 3500);
            } else {
                setTimedStatus(iptvStatus, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(iptvStatus, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveIptv.textContent = "Сохранить параметры IPTV";
        }
    });

    // ----------------------------------------------------
    // 8. SPI Firewall & Security
    // ----------------------------------------------------
    const fwWanPing = document.getElementById("fw-wan-ping");
    const fwSynFlood = document.getElementById("fw-syn-flood");
    const fwPortScan = document.getElementById("fw-port-scan");
    const btnSaveFw = document.getElementById("btn-save-firewall");
    const fwStatus = document.getElementById("fw-save-status");

    fetch("/api/v1/firewall/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (fwWanPing) fwWanPing.checked = !!cfg.wan_ping_respond;
            if (fwSynFlood) fwSynFlood.checked = !!cfg.syn_flood_protection;
            if (fwPortScan) fwPortScan.checked = !!cfg.port_scan_protection;
        })
        .catch(e => console.warn("Firewall load failed", e));

    btnSaveFw?.addEventListener("click", async () => {
        const payload = {
            wan_ping_respond: fwWanPing.checked,
            syn_flood_protection: fwSynFlood.checked,
            port_scan_protection: fwPortScan.checked,
        };
        btnSaveFw.textContent = "Сохранение...";
        try {
            const res = await fetch("/api/v1/firewall/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(fwStatus, "Правила безопасности сохранены", "success", 3500);
            } else {
                setTimedStatus(fwStatus, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(fwStatus, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveFw.textContent = "Сохранить правила безопасности";
        }
    });

    // ----------------------------------------------------
    // 9. Static Routes Table
    // ----------------------------------------------------
    const routesTbody = document.querySelector("#static-routes-table tbody");
    const btnAddRoute = document.getElementById("btn-add-static-route");
    const btnSaveRoutes = document.getElementById("btn-save-routes");
    const routesStatus = document.getElementById("routes-save-status");
    let currentRoutes = [];

    function renderStaticRoutes() {
        if (!routesTbody) return;
        routesTbody.innerHTML = "";
        if (currentRoutes.length === 0) {
            routesTbody.innerHTML = `<tr><td colspan="5" class="text-center muted">Нет статических маршрутов. Нажмите «+ Добавить маршрут».</td></tr>`;
            return;
        }
        currentRoutes.forEach((r, idx) => {
            const tr = document.createElement("tr");
            tr.innerHTML = `
                <td><input type="text" class="form-control text-xs r-dest" value="${r.destination || ''}" placeholder="10.0.0.0/8"></td>
                <td><input type="text" class="form-control text-xs r-gw" value="${r.gateway || ''}" placeholder="192.168.1.254"></td>
                <td>
                    <select class="form-control text-xs r-iface">
                        <option value="WAN" ${r.interface === 'WAN' ? 'selected' : ''}>WAN</option>
                        <option value="LAN" ${r.interface === 'LAN' ? 'selected' : ''}>LAN</option>
                    </select>
                </td>
                <td><input type="number" class="form-control text-xs r-metric" value="${r.metric !== undefined ? r.metric : 1}" style="width: 70px;"></td>
                <td><button class="btn btn-xs btn-outline-danger btn-del-route" data-idx="${idx}">Удалить</button></td>
            `;
            routesTbody.appendChild(tr);
        });

        routesTbody.querySelectorAll(".btn-del-route").forEach(btn => {
            btn.addEventListener("click", (e) => {
                const idx = parseInt(e.target.dataset.idx, 10);
                currentRoutes.splice(idx, 1);
                renderStaticRoutes();
            });
        });
    }

    btnAddRoute?.addEventListener("click", () => {
        currentRoutes.push({ destination: "", gateway: "", interface: "WAN", metric: 1 });
        renderStaticRoutes();
    });

    fetch("/api/v1/routes/config")
        .then(r => r.json())
        .then(routes => {
            currentRoutes = routes || [];
            renderStaticRoutes();
        })
        .catch(e => console.warn("Static routes load failed", e));

    btnSaveRoutes?.addEventListener("click", async () => {
        const rows = routesTbody?.querySelectorAll("tr") || [];
        const routes = [];
        rows.forEach(tr => {
            const dest = tr.querySelector(".r-dest")?.value.trim();
            const gw = tr.querySelector(".r-gw")?.value.trim();
            const iface = tr.querySelector(".r-iface")?.value;
            const metric = parseInt(tr.querySelector(".r-metric")?.value, 10) || 1;
            if (dest && gw) {
                routes.push({ destination: dest, gateway: gw, interface: iface, metric });
            }
        });

        btnSaveRoutes.textContent = "Применение...";
        try {
            const res = await fetch("/api/v1/routes/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(routes),
            });
            const data = await res.json();
            if (data.success) {
                currentRoutes = routes;
                setTimedStatus(routesStatus, "Статические маршруты применены", "success", 3500);
            } else {
                setTimedStatus(routesStatus, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(routesStatus, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveRoutes.textContent = "Применить маршруты";
        }
    });

    // ----------------------------------------------------
    // Hardware & Software Flow Offloading (Fast-Path)
    // ----------------------------------------------------
    const flowHw = document.getElementById("flow-offload-hw");
    const flowSw = document.getElementById("flow-offload-sw");
    const flowBbr = document.getElementById("flow-offload-bbr");
    const flowBadge = document.getElementById("flow-offload-badge");
    const btnSaveOffload = document.getElementById("btn-save-offload");
    const flowStatus = document.getElementById("flow-offload-status");

    fetch("/api/v1/network/offload")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (flowHw) flowHw.checked = !!cfg.hardware_flow_offload;
            if (flowSw) flowSw.checked = !!cfg.software_flow_offload;
            if (flowBbr) flowBbr.checked = !!cfg.tcp_bbr;
            if (flowBadge) {
                if (cfg.hardware_flow_offload) {
                    flowBadge.textContent = "PPE Активен";
                    flowBadge.style.background = "#065f46";
                    flowBadge.style.color = "#34d399";
                } else if (cfg.software_flow_offload) {
                    flowBadge.textContent = "SW FastPath";
                    flowBadge.style.background = "#0369a1";
                    flowBadge.style.color = "#38bdf8";
                } else {
                    flowBadge.textContent = "Отключено";
                    flowBadge.style.background = "#1f293d";
                    flowBadge.style.color = "#94a3b8";
                }
            }
        })
        .catch(e => console.warn("Flow offload load failed", e));

    btnSaveOffload?.addEventListener("click", async () => {
        const payload = {
            software_flow_offload: flowSw ? flowSw.checked : true,
            hardware_flow_offload: flowHw ? flowHw.checked : true,
            tcp_bbr: flowBbr ? flowBbr.checked : true
        };
        btnSaveOffload.textContent = "Применение...";
        try {
            const res = await fetch("/api/v1/network/offload", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(flowStatus, "Сетевое ускорение успешно применено", "success", 3500);
                if (flowBadge) {
                    if (payload.hardware_flow_offload) {
                        flowBadge.textContent = "PPE Активен";
                        flowBadge.style.background = "#065f46";
                        flowBadge.style.color = "#34d399";
                    } else if (payload.software_flow_offload) {
                        flowBadge.textContent = "SW FastPath";
                        flowBadge.style.background = "#0369a1";
                        flowBadge.style.color = "#38bdf8";
                    } else {
                        flowBadge.textContent = "Отключено";
                        flowBadge.style.background = "#1f293d";
                        flowBadge.style.color = "#94a3b8";
                    }
                }
            } else {
                setTimedStatus(flowStatus, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(flowStatus, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveOffload.textContent = "Применить ускорение";
        }
    });
}

// ----------------------------------------------------
// 5. ROG Game Boost & SQM QoS Manager
// ----------------------------------------------------
function initGameBoostManager() {
    const qosBoostEnable = document.getElementById("qos-game-boost");
    const qosSqmEnable = document.getElementById("qos-sqm-enable");
    const qosDownLimit = document.getElementById("qos-down-limit");
    const qosUpLimit = document.getElementById("qos-up-limit");
    const qosGameIps = document.getElementById("qos-game-ips");
    const btnSaveQos = document.getElementById("btn-save-qos");
    const qosStatusSpan = document.getElementById("qos-save-status");
    const trafficTbody = document.querySelector("#traffic-analyzer-table tbody");
    const btnRefreshTraffic = document.getElementById("btn-refresh-traffic");

    fetch("/api/v1/qos/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (qosBoostEnable) qosBoostEnable.checked = !!cfg.game_boost_enabled;
            if (qosSqmEnable) qosSqmEnable.checked = !!cfg.sqm_enabled;
            if (qosDownLimit) qosDownLimit.value = cfg.download_limit_mbps || 95;
            if (qosUpLimit) qosUpLimit.value = cfg.upload_limit_mbps || 95;
            if (qosGameIps && cfg.game_device_ips) qosGameIps.value = cfg.game_device_ips.join(", ");
        })
        .catch(e => console.warn("QoS load failed", e));

    btnSaveQos?.addEventListener("click", async () => {
        const ips = (qosGameIps?.value || "")
            .split(",")
            .map(s => s.trim())
            .filter(Boolean);

        const payload = {
            game_boost_enabled: qosBoostEnable.checked,
            sqm_enabled: qosSqmEnable.checked,
            download_limit_mbps: parseInt(qosDownLimit.value, 10) || 95,
            upload_limit_mbps: parseInt(qosUpLimit.value, 10) || 95,
            game_device_ips: ips,
            prioritize_voice: true,
        };

        btnSaveQos.textContent = "Применение...";
        try {
            const res = await fetch("/api/v1/qos/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(qosStatusSpan, "Параметры SQM QoS применены", "success", 3500);
            } else {
                setTimedStatus(qosStatusSpan, data.message || "Ошибка сохранения", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(qosStatusSpan, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnSaveQos.textContent = "Применить параметры SQM";
        }
    });

    function refreshTrafficAnalyzer() {
        if (!trafficTbody) return;
        fetch("/api/v1/clients/traffic")
            .then(r => r.json())
            .then(stats => {
                trafficTbody.innerHTML = "";
                if (!stats || stats.length === 0) {
                    trafficTbody.innerHTML = `<tr><td colspan="7" class="text-center muted">Нет данных о трафике</td></tr>`;
                    return;
                }
                stats.forEach(c => {
                    const tr = document.createElement("tr");
                    const rxDisplay = c.rx_rate_kbps > 1024 ? (c.rx_rate_kbps / 1024).toFixed(1) + " Мбит/с" : c.rx_rate_kbps + " Кбит/с";
                    const txDisplay = c.tx_rate_kbps > 1024 ? (c.tx_rate_kbps / 1024).toFixed(1) + " Мбит/с" : c.tx_rate_kbps + " Кбит/с";
                    const totalMb = (c.total_rx_mb + c.total_tx_mb).toFixed(1) + " МБ";

                    tr.innerHTML = `
                        <td><strong>${c.hostname || 'Device'}</strong></td>
                        <td class="text-cyan font-bold">${c.ip}</td>
                        <td class="font-mono text-xs">${c.mac}</td>
                        <td class="text-emerald font-bold">${rxDisplay}</td>
                        <td class="text-purple font-bold">${txDisplay}</td>
                        <td class="muted text-xs">${totalMb}</td>
                        <td>${c.is_gaming ? '<span class="badge-pill">SQM Приоритет</span>' : '<span class="muted text-xs">Стандартный</span>'}</td>
                    `;
                    trafficTbody.appendChild(tr);
                });
            })
            .catch(e => console.warn("Traffic stats failed", e));
    }

    btnRefreshTraffic?.addEventListener("click", refreshTrafficAnalyzer);
    refreshTrafficAnalyzer();
}

// ----------------------------------------------------
// 6. Wi-Fi Radar & Spectrum Analyzer Manager
// ----------------------------------------------------
function initRadarManager() {
    const btnScan = document.getElementById("btn-scan-wifi-radar");
    const btnAcs = document.getElementById("btn-trigger-acs");
    const container2g = document.getElementById("radar-2g-congestion");
    const container5g = document.getElementById("radar-5g-congestion");
    const networksTbody = document.querySelector("#radar-networks-table tbody");
    const networksCount = document.getElementById("radar-networks-count");
    const acsAlert = document.getElementById("radar-acs-alert");
    const acsText = document.getElementById("radar-acs-text");

    function renderCongestion(list, container) {
        if (!container) return;
        container.innerHTML = "";
        const grid = document.createElement("div");
        grid.style.display = "grid";
        grid.style.gridTemplateColumns = "repeat(auto-fit, minmax(70px, 1fr))";
        grid.style.gap = "0.5rem";

        list.forEach(item => {
            const card = document.createElement("div");
            card.style.background = "#111726";
            card.style.border = "1px solid #1f293d";
            card.style.borderRadius = "6px";
            card.style.padding = "0.4rem";
            card.style.textAlign = "center";

            let barColor = "var(--emerald)";
            if (item.utilization_pct > 60) barColor = "var(--rose)";
            else if (item.utilization_pct > 25) barColor = "var(--amber)";

            card.innerHTML = `
                <div style="font-weight:700; font-size:0.85rem; color:var(--cyan);">CH ${item.channel}</div>
                <div style="font-size:0.65rem; color:var(--text-muted);">${item.ap_count} сетей</div>
                <div class="progress-bar mt-1" style="height:4px;">
                    <div class="progress-fill" style="width:${item.utilization_pct}%; background:${barColor};"></div>
                </div>
            `;
            grid.appendChild(card);
        });
        container.appendChild(grid);
    }

    async function scanRadar() {
        if (btnScan) btnScan.textContent = "Сканирование...";
        try {
            const res = await fetch("/api/v1/wifi/scan");
            const data = await res.json();
            if (!data) return;

            // Congestion 2.4G & 5G
            if (data.congestion) {
                const cong2g = data.congestion.filter(c => c.band === "2.4 GHz");
                const cong5g = data.congestion.filter(c => c.band === "5 GHz");
                renderCongestion(cong2g, container2g);
                renderCongestion(cong5g, container5g);
            }

            // Neighbor networks table
            if (networksTbody && data.networks) {
                networksTbody.innerHTML = "";
                if (networksCount) networksCount.textContent = `Сетей в радиусе: ${data.networks.length}`;

                data.networks.forEach(net => {
                    const tr = document.createElement("tr");
                    let sigColor = "text-emerald";
                    if (net.signal_dbm < -75) sigColor = "text-rose";
                    else if (net.signal_dbm < -65) sigColor = "text-amber";

                    tr.innerHTML = `
                        <td><strong>${net.ssid || '<Скрытая сеть>'}</strong></td>
                        <td class="font-mono text-xs">${net.bssid}</td>
                        <td><span class="badge-pill">${net.band}</span></td>
                        <td class="text-cyan font-bold">${net.channel}</td>
                        <td>${net.width_mhz} MHz</td>
                        <td class="${sigColor} font-bold">${net.signal_dbm} dBm</td>
                        <td class="text-xs muted">${net.security}</td>
                    `;
                    networksTbody.appendChild(tr);
                });
            }
        } catch (e) {
            console.warn("Wi-Fi scan failed", e);
        } finally {
            if (btnScan) btnScan.textContent = "Сканировать эфир";
        }
    }

    btnScan?.addEventListener("click", scanRadar);
    scanRadar();

    btnAcs?.addEventListener("click", async () => {
        btnAcs.textContent = "Анализ...";
        try {
            const res = await fetch("/api/v1/wifi/acs/trigger", { method: "POST" });
            const data = await res.json();
            if (data.success && acsAlert && acsText) {
                acsText.textContent = data.message;
                acsAlert.classList.remove("d-none");
                scanRadar();
            }
        } catch (e) {
            alert("Ошибка ACS: " + e.message);
        } finally {
            btnAcs.textContent = "Оптимизировать каналы (ACS)";
        }
    });
}

// ----------------------------------------------------
// 8. Tools, WoL, Watchdog & Diagnostics Manager
// ----------------------------------------------------
function initToolsManager() {
    // 8.1. Wake-on-LAN
    const wolMac = document.getElementById("wol-mac-input");
    const btnWol = document.getElementById("btn-send-wol");
    const wolStatus = document.getElementById("wol-status");

    btnWol?.addEventListener("click", async () => {
        const mac = wolMac?.value.trim();
        if (!mac) {
            alert("Пожалуйста, введите MAC-адрес (например AA:BB:CC:DD:EE:FF)");
            return;
        }
        btnWol.textContent = "Отправка...";
        try {
            const res = await fetch("/api/v1/wol/wake", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ mac })
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(wolStatus, "Magic Packet успешно отправлен", "success", 3500);
            } else {
                setTimedStatus(wolStatus, data.message || "Ошибка отправки", "error", 4000);
            }
        } catch (e) {
            setTimedStatus(wolStatus, "Ошибка: " + e.message, "error", 4000);
        } finally {
            btnWol.textContent = "Отправить Magic Packet";
        }
    });

    // 8.2. Ping Watchdog
    const wdEnable = document.getElementById("watchdog-enable");
    const wdIp = document.getElementById("watchdog-ip");
    const wdInterval = document.getElementById("watchdog-interval");
    const wdFails = document.getElementById("watchdog-fails");
    const wdAction = document.getElementById("watchdog-action");
    const btnSaveWd = document.getElementById("btn-save-watchdog");
    const wdStatus = document.getElementById("watchdog-status");

    fetch("/api/v1/watchdog/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (wdEnable) wdEnable.checked = !!cfg.enabled;
            if (wdIp) wdIp.value = cfg.target_ip || "1.1.1.1";
            if (wdInterval) wdInterval.value = cfg.check_interval_sec || 30;
            if (wdFails) wdFails.value = cfg.failure_threshold || 3;
            if (wdAction) wdAction.value = cfg.action || "restart_wan";
        })
        .catch(e => console.warn("Watchdog load failed", e));

    btnSaveWd?.addEventListener("click", async () => {
        const payload = {
            enabled: wdEnable.checked,
            target_ip: wdIp.value.trim(),
            check_interval_sec: parseInt(wdInterval.value, 10) || 30,
            failure_threshold: parseInt(wdFails.value, 10) || 3,
            action: wdAction.value,
        };
        try {
            const res = await fetch("/api/v1/watchdog/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(wdStatus, "Параметры сохранены", "success", 3500);
            }
        } catch (e) {
            setTimedStatus(wdStatus, "Ошибка: " + e.message, "error", 4000);
        }
    });

    // 8.3. DDNS
    const ddnsEnable = document.getElementById("ddns-enable");
    const ddnsService = document.getElementById("ddns-service");
    const ddnsDomain = document.getElementById("ddns-domain");
    const ddnsToken = document.getElementById("ddns-token");
    const btnSaveDdns = document.getElementById("btn-save-ddns");
    const ddnsStatus = document.getElementById("ddns-status");

    fetch("/api/v1/ddns/config")
        .then(r => r.json())
        .then(cfg => {
            if (!cfg) return;
            if (ddnsEnable) ddnsEnable.checked = !!cfg.enabled;
            if (ddnsService) ddnsService.value = cfg.service || "duckdns";
            if (ddnsDomain) ddnsDomain.value = cfg.domain || "";
            if (ddnsToken) ddnsToken.value = cfg.token_or_pass || "";
        })
        .catch(e => console.warn("DDNS load failed", e));

    btnSaveDdns?.addEventListener("click", async () => {
        const payload = {
            enabled: ddnsEnable.checked,
            service: ddnsService.value,
            domain: ddnsDomain.value.trim(),
            token_or_pass: ddnsToken.value.trim(),
        };
        try {
            const res = await fetch("/api/v1/ddns/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                setTimedStatus(ddnsStatus, "Параметры DDNS сохранены", "success", 3500);
            }
        } catch (e) {
            setTimedStatus(ddnsStatus, "Ошибка: " + e.message, "error", 4000);
        }
    });

    // 8.4. Diagnostics
    const diagTool = document.getElementById("diag-tool");
    const diagTarget = document.getElementById("diag-target");
    const btnRunDiag = document.getElementById("btn-run-diag");
    const diagOutput = document.getElementById("diag-output");

    btnRunDiag?.addEventListener("click", async () => {
        const target = diagTarget?.value.trim();
        if (!target) return;
        const tool = diagTool?.value || "ping";

        btnRunDiag.textContent = "Выполнение...";
        diagOutput.textContent = `Запуск ${tool} для ${target}... Пожалуйста, подождите.\n`;

        try {
            const res = await fetch("/api/v1/diagnostics/run", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ tool, target })
            });
            const data = await res.json();
            diagOutput.textContent = data.output || "Нет вывода";
        } catch (e) {
            diagOutput.textContent = "Ошибка выполнения: " + e.message;
        } finally {
            btnRunDiag.textContent = "Выполнить диагностику";
        }
    });

    // ----------------------------------------------------
    // Built-in WAN Speedtest
    // ----------------------------------------------------
    const btnRunSpeedtest = document.getElementById("btn-run-speedtest");
    const stPing = document.getElementById("st-ping");
    const stJitter = document.getElementById("st-jitter");
    const stDownload = document.getElementById("st-download");
    const stUpload = document.getElementById("st-upload");
    const stServer = document.getElementById("st-server");
    const stStatus = document.getElementById("st-status");

    btnRunSpeedtest?.addEventListener("click", async () => {
        btnRunSpeedtest.disabled = true;
        btnRunSpeedtest.textContent = "Тестирование...";
        if (stStatus) stStatus.textContent = "Измерение задержки, скачивания и отдачи...";
        if (stPing) stPing.innerHTML = `-- <span style="font-size:0.875rem; font-weight:normal;">мс</span>`;
        if (stJitter) stJitter.innerHTML = `-- <span style="font-size:0.875rem; font-weight:normal;">мс</span>`;
        if (stDownload) stDownload.innerHTML = `-- <span style="font-size:0.875rem; font-weight:normal;">Мбит/с</span>`;
        if (stUpload) stUpload.innerHTML = `-- <span style="font-size:0.875rem; font-weight:normal;">Мбит/с</span>`;

        try {
            const res = await fetch("/api/v1/diagnostics/speedtest", {
                method: "POST",
                headers: { "Content-Type": "application/json" }
            });
            const data = await res.json();
            if (data && typeof data.ping_ms === "number") {
                if (stPing) stPing.innerHTML = `${data.ping_ms.toFixed(1)} <span style="font-size:0.875rem; font-weight:normal;">мс</span>`;
                if (stJitter) stJitter.innerHTML = `${data.jitter_ms.toFixed(1)} <span style="font-size:0.875rem; font-weight:normal;">мс</span>`;
                if (stDownload) stDownload.innerHTML = `${data.download_mbps.toFixed(2)} <span style="font-size:0.875rem; font-weight:normal;">Мбит/с</span>`;
                if (stUpload) stUpload.innerHTML = `${data.upload_mbps.toFixed(2)} <span style="font-size:0.875rem; font-weight:normal;">Мбит/с</span>`;
                if (stServer) stServer.textContent = "Сервер: " + (data.server_name || "Cloudflare Anycast");
                if (stStatus) stStatus.textContent = "Замер успешно завершен";
            } else {
                if (stStatus) stStatus.textContent = "Ошибка получения результатов замера";
            }
        } catch (e) {
            if (stStatus) stStatus.textContent = "Ошибка: " + e.message;
        } finally {
            btnRunSpeedtest.disabled = false;
            btnRunSpeedtest.textContent = "Запустить тест скорости";
        }
    });
}

function initAuthManager() {
    const modal = document.getElementById("login-modal");
    const form = document.getElementById("login-form");
    const errBox = document.getElementById("login-error");
    const userInput = document.getElementById("login-username");
    const passInput = document.getElementById("login-password");

    const token = localStorage.getItem("ferrum_auth_token");
    if (!token) {
        modal?.classList.remove("d-none");
    }

    form?.addEventListener("submit", async (e) => {
        e.preventDefault();
        errBox?.classList.add("d-none");
        const username = userInput?.value.trim() || "";
        const password = passInput?.value || "";

        try {
            const res = await fetch("/api/v1/auth/login", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username, password })
            });
            const data = await res.json();
            if (data.success && data.token) {
                localStorage.setItem("ferrum_auth_token", data.token);
                modal?.classList.add("d-none");
            } else {
                if (errBox) {
                    errBox.textContent = data.message || "Неверный логин или пароль. По умолчанию: root / root";
                    errBox.classList.remove("d-none");
                }
            }
        } catch (err) {
            if (errBox) {
                errBox.textContent = "Ошибка связи с сервером: " + err.message;
                errBox.classList.remove("d-none");
            }
        }
    });
}

function initSystemManager() {
    const formPw = document.getElementById("form-change-password");
    const statusPw = document.getElementById("change-pw-status");
    formPw?.addEventListener("submit", async (e) => {
        e.preventDefault();
        const old_password = document.getElementById("pw-old")?.value || "";
        const new_password = document.getElementById("pw-new")?.value || "";
        const confirm_password = document.getElementById("pw-confirm")?.value || "";

        if (new_password !== confirm_password) {
            if (statusPw) {
                setTimedStatus(statusPw, "Новые пароли не совпадают", "error", 3500);
            }
            return;
        }

        try {
            const res = await fetch("/api/v1/auth/change_password", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ old_password, new_password })
            });
            const data = await res.json();
            if (data.success) {
                if (statusPw) {
                    setTimedStatus(statusPw, data.message || "Пароль успешно обновлен", "success", 3500);
                }
                formPw.reset();
            } else {
                if (statusPw) {
                    setTimedStatus(statusPw, data.message || "Ошибка смены пароля", "error", 4000);
                }
            }
        } catch (err) {
            if (statusPw) {
                setTimedStatus(statusPw, "Ошибка: " + err.message, "error", 4000);
            }
        }
    });
}

// ----------------------------------------------------
// Firmware Upgrade Management (.trx / MTD Flash)
// ----------------------------------------------------
function initFirmwareUpgrade() {
    const fileInput = document.getElementById("fw-file-input");
    const detailsBox = document.getElementById("fw-file-details");
    const fwName = document.getElementById("fw-name");
    const fwSize = document.getElementById("fw-size");
    const fwCheck = document.getElementById("fw-check");
    const btnFlash = document.getElementById("btn-flash-fw");
    const progressWrap = document.getElementById("fw-progress-wrap");
    const progressBar = document.getElementById("fw-progress-bar");
    const progressPercent = document.getElementById("fw-progress-percent");
    const stepLabel = document.getElementById("fw-step-label");
    const alertBox = document.getElementById("fw-alert");
    const statusLabel = document.getElementById("fw-action-status");

    let selectedFile = null;

    fileInput?.addEventListener("change", (e) => {
        const file = e.target.files?.[0];
        if (!file) {
            selectedFile = null;
            if (detailsBox) detailsBox.style.display = "none";
            if (btnFlash) btnFlash.disabled = true;
            return;
        }

        selectedFile = file;
        const sizeMb = (file.size / (1024 * 1024)).toFixed(2);

        if (fwName) fwName.textContent = file.name;
        if (fwSize) fwSize.textContent = `${file.size.toLocaleString()} байт (${sizeMb} МБ)`;

        if (alertBox) alertBox.style.display = "none";

        // Pre-validation checks
        if (file.size > 8060928) {
            if (fwCheck) {
                fwCheck.textContent = "Превышен размер MTD Flash (> 7.68 МБ)";
                fwCheck.className = "text-red";
            }
            if (btnFlash) btnFlash.disabled = true;
            if (alertBox) {
                alertBox.textContent = `Ошибка: Файл ${file.name} (${sizeMb} МБ) превышает размер доступного MTD-раздела Flash-памяти (7.68 МБ).`;
                alertBox.style.display = "block";
            }
            return;
        }

        if (file.size < 1000000) {
            if (fwCheck) {
                fwCheck.textContent = "Файл слишком мал (< 1 МБ)";
                fwCheck.className = "text-red";
            }
            if (btnFlash) btnFlash.disabled = true;
            return;
        }

        if (fwCheck) {
            fwCheck.textContent = "Готов к прошивке (Размер в норме)";
            fwCheck.className = "text-green";
        }
        if (detailsBox) detailsBox.style.display = "block";
        if (btnFlash) btnFlash.disabled = false;
    });

    btnFlash?.addEventListener("click", async () => {
        if (!selectedFile) return;

        if (!confirm(`Вы действительно хотите прошить образ ${selectedFile.name}?\n\nВнимание: Не отключайте питание роутера во время записи во Flash.`)) {
            return;
        }

        btnFlash.disabled = true;
        if (fileInput) fileInput.disabled = true;
        if (progressWrap) progressWrap.style.display = "block";
        if (alertBox) alertBox.style.display = "none";

        try {
            // Step 1: Uploading binary via XMLHttpRequest for progress monitoring
            if (stepLabel) stepLabel.textContent = "Этап 1/3: Передача образа в оперативную память роутера...";
            
            await new Promise((resolve, reject) => {
                const xhr = new XMLHttpRequest();
                xhr.open("POST", "/api/v1/system/firmware/upload", true);

                xhr.upload.onprogress = (evt) => {
                    if (evt.lengthComputable) {
                        const pct = Math.round((evt.loaded / evt.total) * 100);
                        if (progressBar) progressBar.style.width = pct + "%";
                        if (progressPercent) progressPercent.textContent = pct + "%";
                    }
                };

                xhr.onload = () => {
                    if (xhr.status >= 200 && xhr.status < 300) {
                        try {
                            const res = JSON.parse(xhr.responseText);
                            if (res.success) {
                                resolve(res);
                            } else {
                                reject(new Error(res.message || "Ошибка проверки заголовка прошивки"));
                            }
                        } catch (err) {
                            reject(new Error("Некорректный ответ от роутера"));
                        }
                    } else {
                        reject(new Error(`Ошибка HTTP ${xhr.status}: ${xhr.statusText}`));
                    }
                };

                xhr.onerror = () => reject(new Error("Сетевой сбой при передаче файла"));
                xhr.send(selectedFile);
            });

            // Step 2: Triggering Flash write
            if (stepLabel) stepLabel.textContent = "Этап 2/3: Проверка целостности и запуск mtd_write в SPI Flash...";
            if (progressBar) progressBar.style.width = "100%";
            if (progressPercent) progressPercent.textContent = "100%";

            const flashRes = await fetch("/api/v1/system/firmware/flash", { method: "POST" });
            const flashData = await flashRes.json();

            if (!flashData.success) {
                throw new Error(flashData.message || "Не удалось запустить процесс прошивки");
            }

            // Step 3: Countdown and Auto-Reconnect
            let countdown = 45;
            if (stepLabel) stepLabel.textContent = `Этап 3/3: Запись во Flash! Роутер перезагружается (${countdown} сек)...`;
            if (statusLabel) statusLabel.textContent = "Пожалуйста, подождите. Веб-панель перезагрузится автоматически.";

            const timer = setInterval(() => {
                countdown--;
                if (stepLabel) stepLabel.textContent = `Этап 3/3: Роутер перезагружается... Ожидание (${countdown} сек)`;
                if (countdown <= 0) {
                    clearInterval(timer);
                    if (stepLabel) stepLabel.textContent = "Попытка переподключения к панели управления...";
                    
                    const checkInterval = setInterval(async () => {
                        try {
                            const ping = await fetch("/httpd_check.htm", { cache: "no-store" });
                            if (ping.ok) {
                                clearInterval(checkInterval);
                                location.reload();
                            }
                        } catch (_) {}
                    }, 2000);
                }
            }, 1000);

        } catch (err) {
            if (alertBox) {
                alertBox.textContent = "Ошибка обновления: " + err.message;
                alertBox.style.display = "block";
            }
            if (btnFlash) btnFlash.disabled = false;
            if (fileInput) fileInput.disabled = false;
            if (stepLabel) stepLabel.textContent = "Ошибка при обновлении прошивки";
        }
    });
}

// Global Show/Hide Password Toggle
document.addEventListener("click", (e) => {
    const btn = e.target.closest(".btn-toggle-password");
    if (!btn) return;
    const targetId = btn.getAttribute("data-target");
    const input = document.getElementById(targetId);
    if (!input) return;
    if (input.type === "password") {
        input.type = "text";
        btn.textContent = "🙈";
    } else {
        input.type = "password";
        btn.textContent = "👁️";
    }
});
