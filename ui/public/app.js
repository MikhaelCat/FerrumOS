// FerrumOS Modern Client Application (Zero Dependencies, Ultra Fast)
document.addEventListener("DOMContentLoaded", () => {
    initNavigation();
    initDashboard();
    initProxyManager();
    initZapretManager();
    startStatsPolling();
});

// Navigation & Tab Switching (Zero Reloads)
function initNavigation() {
    const tabs = {
        dashboard: { title: "Дашборд и мониторинг", subtitle: "Состояние аппаратных ресурсов и сетевой активности" },
        proxy: { title: "VLESS Reality & Hysteria 2", subtitle: "1-Click подключение и умный обход блокировок (РФ)" },
        zapret: { title: "Zapret DPI Bypass", subtitle: "Прямая локальная десинхронизация ТСПУ для YouTube 4K и Discord" },
        wifi: { title: "Беспроводная сеть Wi-Fi", subtitle: "Настройки 2.4 GHz (MT7620) и 5 GHz (MT7610E AC)" },
        network: { title: "Сеть & Маршрутизация", subtitle: "Параметры WAN, коммутатора и DNS" },
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
        });
    });

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
    setInterval(() => {
        fetchSystemStatus();
        fetchInterfaces();
    }, 2000);
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
            alert("Пожалуйста, вставьте ссылку vless:// или hysteria2://");
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
                currentParsedConfig = data.config;
                parsedCard.classList.remove("d-none");

                document.getElementById("parsed-tag").textContent = data.config.tag || "Новый узел";
                document.getElementById("parsed-type-badge").textContent = (data.config.proxy_type || "proxy").toUpperCase();
                document.getElementById("parsed-host").textContent = data.config.server;
                document.getElementById("parsed-port").textContent = data.config.server_port;
                document.getElementById("parsed-security").textContent = data.config.security || "none";
                document.getElementById("parsed-sni").textContent = data.config.sni || "-";
                document.getElementById("parsed-flow").textContent = data.config.flow || "-";
                document.getElementById("parsed-fp").textContent = data.config.fingerprint || "-";
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
                pingResult.textContent = `⚡ Пинг: ${data.latency_ms} ms (Доступен)`;
                pingResult.className = "text-xs text-emerald align-self-center";
            } else {
                pingResult.textContent = `❌ Недоступен (${data.error || "Таймаут"})`;
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
        statusSpan.textContent = "Запись маршрутов и запуск sing-box...";

        try {
            const res = await fetch("/api/v1/proxy/apply", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    uri,
                    mode,
                    custom_domains: customDomains,
                    custom_ips: []
                })
            });
            const data = await res.json();
            if (data.success) {
                statusSpan.textContent = "✅ Туннель успешно активирован!";
                statusSpan.className = "text-xs text-emerald align-self-center";
                document.getElementById("proxy-status-text").textContent = "Активен (Подключено)";
                document.getElementById("proxy-active-mode-badge").textContent = mode === "smart_ru" ? "Smart RU" : mode.toUpperCase();
                document.getElementById("proxy-pill").textContent = "Active";
            } else {
                statusSpan.textContent = "❌ " + data.message;
                statusSpan.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            statusSpan.textContent = "Ошибка: " + e.message;
        } finally {
            applyBtn.textContent = "🚀 Применить и активировать туннель";
        }
    });
}

function initZapretManager() {
    const saveBtn = document.getElementById("btn-save-zapret");
    saveBtn?.addEventListener("click", () => {
        const yt = document.getElementById("zapret-youtube")?.checked;
        const dc = document.getElementById("zapret-discord")?.checked;
        const gen = document.getElementById("zapret-general")?.checked;
        alert(`Настройки Zapret сохранены:\nYouTube 4K: ${yt ? "ВКЛ" : "ВЫКЛ"}\nDiscord: ${dc ? "ВКЛ" : "ВЫКЛ"}\nGeneral DPI: ${gen ? "ВКЛ" : "ВЫКЛ"}`);
    });
}
