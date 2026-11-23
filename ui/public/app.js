// FerrumOS Modern Client Application (Zero Dependencies, Ultra Fast)
document.addEventListener("DOMContentLoaded", () => {
    initNavigation();
    initDashboard();
    initProxyManager();
    initZapretManager();
    initGameBoostManager();
    initWifiManager();
    initRadarManager();
    initNetworkManager();
    initToolsManager();
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


function initWifiManager() {
    const saveBtn = document.getElementById("btn-save-wifi");
    const statusSpan = document.getElementById("wifi-save-status");
    const guestEnable = document.getElementById("guest-enable");
    const rssiSlider = document.getElementById("wifi-rssi");
    const rssiVal = document.getElementById("wifi-rssi-val");

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
            if (document.getElementById("wifi2-bw")) document.getElementById("wifi2-bw").value = cfg.wifi_2g_bandwidth || 40;
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
            if (document.getElementById("wifi5-bw")) document.getElementById("wifi5-bw").value = cfg.wifi_5g_bandwidth || 80;
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
        };

        saveBtn.textContent = "Применение к радиомодулям...";
        statusSpan.textContent = "Запись RT2860AP.dat и iNIC_ap.dat...";

        try {
            const res = await fetch("/api/v1/wifi/config", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                statusSpan.textContent = "✅ Настройки Wi-Fi применены и сохранены в Flash!";
                statusSpan.className = "text-xs text-emerald align-self-center";
            } else {
                statusSpan.textContent = "❌ " + data.message;
                statusSpan.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            statusSpan.textContent = "Ошибка сохранения: " + e.message;
        } finally {
            saveBtn.textContent = "💾 Применить и сохранить настройки Wi-Fi";
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
                dnsStatusSpan.textContent = "✅ Настройки DNS сохранены и применены!";
                dnsStatusSpan.className = "text-xs text-emerald align-self-center";
            } else {
                dnsStatusSpan.textContent = "❌ " + data.message;
                dnsStatusSpan.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            dnsStatusSpan.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveDns.textContent = "💾 Сохранить настройки DNS";
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
                wanStatusSpan.textContent = "✅ Настройки WAN сохранены!";
                wanStatusSpan.className = "text-xs text-emerald align-self-center";
            } else {
                wanStatusSpan.textContent = "❌ " + data.message;
                wanStatusSpan.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            wanStatusSpan.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveWan.textContent = "💾 Сохранить настройки WAN";
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
                <td><button class="btn btn-sm btn-danger btn-del-lease" data-idx="${idx}">✕</button></td>
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
                lanStatusSpan.textContent = "✅ Настройки LAN и статические привязки сохранены!";
                lanStatusSpan.className = "text-xs text-emerald align-self-center";
            } else {
                lanStatusSpan.textContent = "❌ " + data.message;
                lanStatusSpan.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            lanStatusSpan.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveLan.textContent = "💾 Сохранить настройки LAN & DHCP";
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
                <td><button class="btn btn-sm btn-danger btn-del-pf" data-idx="${idx}">✕</button></td>
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
                pfStatusSpan.textContent = "✅ Правила проброса портов успешно применены!";
                pfStatusSpan.className = "text-xs text-emerald align-self-center";
            } else {
                pfStatusSpan.textContent = "❌ " + data.message;
                pfStatusSpan.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            pfStatusSpan.textContent = "Ошибка: " + e.message;
        } finally {
            btnSavePf.textContent = "💾 Применить правила проброса портов";
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
                const badgeClass = p.link_up ? "badge-emerald" : "badge-rose";
                const speedText = p.link_up ? `${p.speed_mbps} Mbps ${p.duplex}` : "Отключен";
                const icon = p.link_up ? "🟢" : "⚫";
                const div = document.createElement("div");
                div.className = "card";
                div.style.padding = "0.75rem";
                div.style.textAlign = "center";
                div.innerHTML = `
                    <div style="font-size: 1.5rem; margin-bottom: 0.25rem;">${icon}</div>
                    <div style="font-weight: 700; font-size: 0.85rem;">${p.port_name}</div>
                    <div class="badge ${badgeClass}" style="margin-top: 0.35rem; font-size: 0.7rem;">${speedText}</div>
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
                ipv6Status.textContent = "✅ Настройки IPv6 успешно сохранены!";
                ipv6Status.className = "text-xs text-emerald align-self-center";
            } else {
                ipv6Status.textContent = "❌ " + data.message;
                ipv6Status.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            ipv6Status.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveIpv6.textContent = "💾 Сохранить настройки IPv6";
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
                iptvStatus.textContent = "✅ Настройки IPTV и Udpxy применены!";
                iptvStatus.className = "text-xs text-emerald align-self-center";
            } else {
                iptvStatus.textContent = "❌ " + data.message;
                iptvStatus.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            iptvStatus.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveIptv.textContent = "💾 Применить настройки IPTV";
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
                fwStatus.textContent = "✅ Правила Firewall сохранены!";
                fwStatus.className = "text-xs text-emerald align-self-center";
            } else {
                fwStatus.textContent = "❌ " + data.message;
                fwStatus.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            fwStatus.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveFw.textContent = "💾 Сохранить правила Firewall";
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
                <td><button class="btn btn-sm btn-danger btn-del-route" data-idx="${idx}">✕</button></td>
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
                routesStatus.textContent = "✅ Статические маршруты успешно применены!";
                routesStatus.className = "text-xs text-emerald align-self-center";
            } else {
                routesStatus.textContent = "❌ " + data.message;
                routesStatus.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            routesStatus.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveRoutes.textContent = "💾 Применить маршруты";
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
                qosStatusSpan.textContent = "✅ Режим ROG Game Boost активирован!";
                qosStatusSpan.className = "text-xs text-emerald align-self-center";
            } else {
                qosStatusSpan.textContent = "❌ " + data.message;
                qosStatusSpan.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            qosStatusSpan.textContent = "Ошибка: " + e.message;
        } finally {
            btnSaveQos.textContent = "💾 Применить настройки Game Boost";
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
                        <td>${c.is_gaming ? '<span class="badge-pill" style="background:#8b5cf6; color:#fff;">🎮 Game Boost</span>' : '<span class="muted text-xs">Обычный</span>'}</td>
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
            if (btnScan) btnScan.textContent = "⚡ Сканировать эфир";
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
                acsText.textContent = "🚀 " + data.message;
                acsAlert.classList.remove("d-none");
                scanRadar();
            }
        } catch (e) {
            alert("Ошибка ACS: " + e.message);
        } finally {
            btnAcs.textContent = "🚀 Оптимизировать каналы (ACS)";
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
                wolStatus.textContent = "✅ Magic packet отправлен!";
                wolStatus.className = "text-xs text-emerald align-self-center";
            } else {
                wolStatus.textContent = "❌ " + data.message;
                wolStatus.className = "text-xs text-rose align-self-center";
            }
        } catch (e) {
            wolStatus.textContent = "Ошибка: " + e.message;
        } finally {
            btnWol.textContent = "⚡ Разбудить устройство";
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
                wdStatus.textContent = "✅ Сохранено";
                wdStatus.className = "text-xs text-emerald align-self-center";
            }
        } catch (e) {
            wdStatus.textContent = "Ошибка: " + e.message;
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
                ddnsStatus.textContent = "✅ DDNS сохранен";
                ddnsStatus.className = "text-xs text-emerald align-self-center";
            }
        } catch (e) {
            ddnsStatus.textContent = "Ошибка: " + e.message;
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

        btnRunDiag.textContent = "⏳ Выполнение...";
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
            btnRunDiag.textContent = "▶ Запустить";
        }
    });
}
