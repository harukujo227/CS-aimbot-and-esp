let ws = null;

const MapData = {
    de_ancient_night: { x: -2953, y: 2164, scale: 4.4, rotate: false, zoom: 1 },
    de_dust2: { x: -2476, y: 3239, scale: 4.4, rotate: true, zoom: 1.1 },
    de_inferno: { x: -2087, y: 3870, scale: 4.9, rotate: false, zoom: 1 },
    de_mirage: { x: -3230, y: 1713, scale: 5, rotate: false, zoom: 1 },
    de_nuke: { x: -3453, y: 2887, scale: 7, rotate: false, zoom: 1, lowerThreshold: -495 },
    de_overpass: { x: -4831, y: 1781, scale: 5.2, rotate: false, zoom: 1 },
    de_vertigo: { x: -3168, y: 1762, scale: 4, rotate: false, zoom: 1, lowerThreshold: 11700 },
};

window.radarConfig = {
    enemy_dot_health_based: true,
    enemy_dot_color: [255, 68, 68],
    show_teammates: true,
    teammate_dot_color: [93, 156, 236]
};

const loc = window.location;
const uuid = new URLSearchParams(loc.search).get("uuid");
const url = `ws://${loc.hostname}:${loc.port}`;

let clientId = localStorage.getItem('radarClientId');
if (!clientId) {
    clientId = 'client_' + Math.random().toString(36).substr(2, 9) + '_' + Date.now();
    localStorage.setItem('radarClientId', clientId);
}

let players = [];
let friendlies = [];
let bombData = null;
let map_info = MapData["de_mirage"];
let currentMapName = "de_mirage";

let scale = 1;
let panX = 0;
let panY = 0;
let isPanning = false;
let startX = 0;
let startY = 0;

let currentRound = 1;
let roundTime = 115;
let terroristScore = 0;
let ctScore = 0;

let focusedPlayer = null;
let isTrackingPlayer = false;
let trackingScale = 2;

let initialDistance = 0;
let initialScale = 1;
let initialTouchX = 0;
let initialTouchY = 0;

let viewerCount = 0;
let isRegisteredViewer = false;

const radar = document.querySelector('.radar');
const radarContent = document.querySelector('.radar-content');
const radarBackground = document.getElementById("radar-background");

if (radarBackground) {
    radarBackground.src = `/radars/de_mirage.png`;
}

function startWebSocket() {
    stopWebSocket();

    console.log("Opening WebSocket connection...");

    ws = new WebSocket(url);
    ws.onmessage = wsMessage;
    ws.onopen = () => {
        console.log('WebSocket connected successfully');
        updateConnectionStatus(true);

        if (uuid && !isRegisteredViewer) {
            setTimeout(registerViewer, 100);
        }
    };
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateConnectionStatus(false);
    };
    ws.onclose = (event) => {
        console.log('WebSocket closed:', event.code, event.reason);
        updateConnectionStatus(false);
        isRegisteredViewer = false;
    };
}


function stopWebSocket() {
    if (ws) {
        ws.close(1000, 'Manual close');
        ws = null;
    }
    isRegisteredViewer = false;
}

function wsMessage(event) {
    let jsonData;

    if (typeof event.data === 'string') {
        jsonData = event.data;

        if (jsonData.length === 0) {
            return;
        }

        try {
            const json = JSON.parse(jsonData);

            if (json.kind === "viewer_count") {
                updateViewerCount(json.count);
                return;
            }

            processRadarData(json);
        } catch (error) {
            console.error('Error parsing text JSON:', error);
            console.log('Raw data:', jsonData.substring(0, 200) + '...');
        }

    } else if (event.data instanceof Blob) {

        if (event.data.size === 0) {
            console.warn('Empty blob received');
            return;
        }

        const reader = new FileReader();
        reader.onload = function() {
            const text = reader.result;

            if (!text || text.length === 0) {
                console.warn('Empty text from blob');
                return;
            }

            try {
                const json = JSON.parse(text);

                if (json.kind === "viewer_count") {
                    updateViewerCount(json.count);
                    return;
                }

                processRadarData(json);
            } catch (error) {
                console.error('Error parsing blob JSON:', error);
                console.log('Blob text:', text.substring(0, 200) + '...');
            }
        };
        reader.onerror = function() {
            console.error('Error reading blob');
        };
        reader.readAsText(event.data);

    } else {
        console.error('Unknown message type:', typeof event.data, event.data);
    }
}

function registerViewer() {
    if (ws && ws.readyState === WebSocket.OPEN && uuid && !isRegisteredViewer) {
        const message = JSON.stringify({
            kind: "register_viewer",
            uuid: uuid,
            client_id: clientId
        });

        try {
            ws.send(message);
            isRegisteredViewer = true;
            console.log('Registered as viewer for game:', uuid, 'with client ID:', clientId);
        } catch (error) {
            console.error('Error registering as viewer:', error);
        }
    }
}

function updateConnectionStatus(connected) {
    const indicator = document.getElementById('connectionIndicator');
    const text = document.getElementById('connectionText');

    if (connected) {
        indicator?.classList.add('online');
        indicator?.classList.remove('offline');
        if (text) text.textContent = 'Connected';
    } else {
        indicator?.classList.add('offline');
        indicator?.classList.remove('online');
        if (text) text.textContent = 'Disconnected';
    }
}

function updateViewerCount(count) {
    viewerCount = count;
    const viewerCountElement = document.getElementById('viewerCount');
    if (viewerCountElement) {
        viewerCountElement.textContent = count;
    }
}

function processRadarData(json) {
    players = json.players;
    friendlies = json.friendlies;
    bombData = json.bomb;

    //WILL WORK IN THE FUTURE
    if (json.round_number) {
        currentRound = json.round_number;
        const roundElement = document.getElementById("roundNumber");
        if (roundElement) roundElement.textContent = currentRound;
    }

    if (json.round_time !== undefined) {
        roundTime = json.round_time;
        updateRoundTimer();
    }

    if (json.terrorist_score !== undefined) {
        terroristScore = json.terrorist_score;
        const terroristElement = document.getElementById("terroristScore");
        if (terroristElement) terroristElement.textContent = terroristScore;
    }

    if (json.ct_score !== undefined) {
        ctScore = json.ct_score;
        const ctElement = document.getElementById("ctScore");
        if (ctElement) ctElement.textContent = ctScore;
    }

    if (json.map_name && json.map_name !== currentMapName) {
        currentMapName = json.map_name;
        if (radarBackground) {
            radarBackground.src = `/radars/${json.map_name}.png`;
        }
        map_info = MapData[json.map_name] || MapData["de_mirage"];

        const mapNameElement = document.getElementById("mapName");
        if (mapNameElement) {
            mapNameElement.textContent = json.map_name.replace("de_", "").toUpperCase();
        }

        initializeRadarBackground();
    }

    addPlayers();
    updatePlayerLists();
}

document.addEventListener('DOMContentLoaded', () => {
    initializeRadarBackground();
    setupZoomControls();
    setupMouseEvents();
    setupTouchEvents();
    setupKeyboardShortcuts();

    const settingsPanel = document.getElementById("settingsPanel");
    const settingsToggle = document.getElementById("settingsToggle");
    const closeSettings = document.getElementById("closeSettings");

    settingsToggle.addEventListener("click", () => {
        settingsPanel.classList.toggle("hidden");
    });
    closeSettings.addEventListener("click", () => {
        settingsPanel.classList.add("hidden");
    });

    let isDragging = false;
    let offsetX = 0, offsetY = 0;

    settingsPanel.addEventListener("mousedown", (e) => {
        if (e.target.tagName === "INPUT" || e.target.tagName === "BUTTON" || e.target.tagName === "LABEL") return;
        isDragging = true;
        offsetX = e.clientX - settingsPanel.offsetLeft;
        offsetY = e.clientY - settingsPanel.offsetTop;
        settingsPanel.style.transition = "none";
    });

    document.addEventListener("mousemove", (e) => {
        if (isDragging) {
            settingsPanel.style.left = `${e.clientX - offsetX}px`;
            settingsPanel.style.top = `${e.clientY - offsetY}px`;
            settingsPanel.style.bottom = "auto";
        }
    });

    document.addEventListener("mouseup", () => {
        isDragging = false;
        settingsPanel.style.transition = "";
    });

    loadSettings();

    ["showTeammates", "enemyHpColor", "enemyColor", "teammateColor"].forEach(id => {
        document.getElementById(id).addEventListener("change", saveSettings);
    });
});


function loadSettings() {
    const saved = JSON.parse(localStorage.getItem("radarSettings")) || {};

    const defaults = {
        showTeammates: true,
        enemyHpColor: true,
        enemyColor: "#ff4444",
        teammateColor: "#5d9cec",
    };

    const merged = { ...defaults, ...saved };

    document.getElementById("showTeammates").checked = merged.showTeammates;
    document.getElementById("enemyHpColor").checked = merged.enemyHpColor;
    document.getElementById("enemyColor").value = merged.enemyColor;
    document.getElementById("teammateColor").value = merged.teammateColor;

    applySettings(merged);
}

function saveSettings() {
    const settings = {
        showTeammates: document.getElementById("showTeammates")?.checked ?? true,
        enemyHpColor: document.getElementById("enemyHpColor")?.checked ?? true,
        enemyColor: document.getElementById("enemyColor")?.value ?? "#ff4444",
        teammateColor: document.getElementById("teammateColor")?.value ?? "#5d9cec",
    };
    localStorage.setItem("radarSettings", JSON.stringify(settings));
    applySettings(settings);
}

function applySettings(settings) {
    if (!window.radarConfig) window.radarConfig = {};

    window.radarConfig.show_teammates = settings.showTeammates;
    window.radarConfig.enemy_dot_health_based = settings.enemyHpColor;
    window.radarConfig.enemy_dot_color = hexToRgb(settings.enemyColor, [255, 68, 68]);
    window.radarConfig.teammate_dot_color = hexToRgb(settings.teammateColor, [93, 156, 236]);

    addPlayers();
    updatePlayerLists();
}

function hexToRgb(hex, fallback = [255, 255, 255]) {
    if (!hex || typeof hex !== "string") return fallback;

    hex = hex.replace("#", "");
    if (hex.length !== 6) return fallback;

    const bigint = parseInt(hex, 16);
    return [
        (bigint >> 16) & 255,
        (bigint >> 8) & 255,
        bigint & 255,
    ];
}

function setupZoomControls() {
    const zoomInBtn = document.getElementById('zoomIn');
    const zoomOutBtn = document.getElementById('zoomOut');
    const resetZoomBtn = document.getElementById('resetZoom');

    zoomInBtn?.addEventListener('click', () => {
        if (isTrackingPlayer) {
            stopTracking();
        } else {
            scale = Math.min(3, scale * 1.2);
            updateRadarTransform();
        }
    });

    zoomOutBtn?.addEventListener('click', () => {
        if (isTrackingPlayer) {
            stopTracking();
        } else {
            scale = Math.max(0.5, scale / 1.2);
            updateRadarTransform();
        }
    });

    resetZoomBtn?.addEventListener('click', () => {
        stopTracking();
    });
}

function setupMouseEvents() {
    if (!radar) return;

    radar.addEventListener('wheel', (e) => {
        if (isTrackingPlayer) return;
        e.preventDefault();

        const rect = radar.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        const wheel = e.deltaY < 0 ? 1 : -1;
        const zoom = Math.exp(wheel * 0.1);
        const newScale = Math.min(Math.max(0.5, scale * zoom), 3);

        if (newScale !== scale) {
            const zoomPointX = (mouseX - panX) / scale;
            const zoomPointY = (mouseY - panY) / scale;

            scale = newScale;
            panX = mouseX - zoomPointX * scale;
            panY = mouseY - zoomPointY * scale;

            updateRadarTransform();
        }
    });

    radar.addEventListener('mousedown', (e) => {
        if (e.button !== 0) return;

        const clickedPlayer = e.target.closest('.player-dot, .player-name');
        if (clickedPlayer) return;

        if (isTrackingPlayer) {
            stopTracking();
            return;
        }

        isPanning = true;
        startX = e.clientX;
        startY = e.clientY;

        radar.classList.add('panning');
        radarContent?.classList.add('no-transition');
        e.preventDefault();

        radar._initialPanX = panX;
        radar._initialPanY = panY;
    });

    document.addEventListener('mousemove', (e) => {
        if (isPanning && !isTrackingPlayer) {
            const deltaX = e.clientX - startX;
            const deltaY = e.clientY - startY;

            panX = radar._initialPanX + deltaX;
            panY = radar._initialPanY + deltaY;

            updateRadarTransform();
        }
    });

    document.addEventListener('mouseup', () => {
        if (isPanning) {
            isPanning = false;
            radar?.classList.remove('panning');
            radarContent?.classList.remove('no-transition');
            delete radar?._initialPanX;
            delete radar?._initialPanY;
        }
    });

    radar.style.cursor = 'grab';
}

function setupTouchEvents() {
    if (!radar) return;

    let touchStartTime = 0;
    let touchMoved = false;
    let lastTap = 0;

    radar.addEventListener('touchstart', (e) => {
        touchStartTime = Date.now();
        touchMoved = false;
        radarContent?.classList.add('no-transition');

        if (e.touches.length === 2) {
            const touch1 = e.touches[0];
            const touch2 = e.touches[1];
            initialDistance = Math.hypot(
                touch2.clientX - touch1.clientX,
                touch2.clientY - touch1.clientY
            );
            initialScale = scale;

            const rect = radar.getBoundingClientRect();
            initialTouchX = ((touch1.clientX + touch2.clientX) / 2) - rect.left;
            initialTouchY = ((touch1.clientY + touch2.clientY) / 2) - rect.top;
            e.preventDefault();
        } else if (e.touches.length === 1) {
            const touch = e.touches[0];
            startX = touch.clientX;
            startY = touch.clientY;
            radar._initialPanX = panX;
            radar._initialPanY = panY;

            const target = document.elementFromPoint(touch.clientX, touch.clientY);
            if (target && target.closest('.player-dot, .player-name')) {
                return;
            }

            isPanning = true;
            e.preventDefault();
        }
    });

    radar.addEventListener('touchmove', (e) => {
        touchMoved = true;

        if (e.touches.length === 2) {
            const touch1 = e.touches[0];
            const touch2 = e.touches[1];
            const distance = Math.hypot(
                touch2.clientX - touch1.clientX,
                touch2.clientY - touch1.clientY
            );

            const newScale = Math.min(Math.max(0.5, initialScale * (distance / initialDistance)), 3);

            if (newScale !== scale) {
                const zoomPointX = (initialTouchX - panX) / scale;
                const zoomPointY = (initialTouchY - panY) / scale;

                scale = newScale;
                panX = initialTouchX - zoomPointX * scale;
                panY = initialTouchY - zoomPointY * scale;

                updateRadarTransform();
            }
            e.preventDefault();
        } else if (e.touches.length === 1 && isPanning) {
            const touch = e.touches[0];
            const deltaX = touch.clientX - startX;
            const deltaY = touch.clientY - startY;

            panX = radar._initialPanX + deltaX;
            panY = radar._initialPanY + deltaY;

            updateRadarTransform();
            e.preventDefault();
        }
    });

    radar.addEventListener('touchend', (e) => {
        const touchDuration = Date.now() - touchStartTime;

        if (e.touches.length === 0) {
            if (!touchMoved && touchDuration < 200) {
                const currentTime = Date.now();
                const touch = e.changedTouches[0];
                const target = document.elementFromPoint(touch.clientX, touch.clientY);

                if (target) {
                    const playerElement = target.closest('.player-dot, .player-name');
                    if (playerElement) {
                        const allPlayers = [...(players || []), ...(friendlies || [])];
                        const playerName = playerElement.querySelector('.player-name')?.textContent ||
                        playerElement.textContent ||
                        playerElement.closest('.player-name')?.textContent;

                        if (playerName) {
                            const player = allPlayers.find(p => p.name === playerName.trim());
                            if (player) {
                                focusOnPlayer(player);
                                e.preventDefault();
                                return;
                            }
                        }
                    }
                }

                if (isTrackingPlayer) {
                    stopTracking();
                }
            }

            isPanning = false;
            initialDistance = 0;
            radarContent?.classList.remove('no-transition');
            delete radar?._initialPanX;
            delete radar?._initialPanY;
        }
    });
}

function setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        if (e.target.tagName.toLowerCase() === 'input') return;

        switch(e.key) {
            case '=':
            case '+':
                if (isTrackingPlayer) {
                    stopTracking();
                } else {
                    scale = Math.min(3, scale * 1.2);
                    updateRadarTransform();
                }
                e.preventDefault();
                break;
            case '-':
                if (isTrackingPlayer) {
                    stopTracking();
                } else {
                    scale = Math.max(0.5, scale / 1.2);
                    updateRadarTransform();
                }
                e.preventDefault();
                break;
            case '0':
            case 'Escape':
            case ' ':
                if (isTrackingPlayer) {
                    stopTracking();
                    e.preventDefault();
                }
                break;
        }
    });
}

function initializeRadarBackground() {
    if (radarBackground && radarContent) {
        radarBackground.remove();
        radarContent.appendChild(radarBackground);

        Object.assign(radarBackground.style, {
            position: 'absolute',
            top: '0',
            left: '0',
            width: '100%',
            height: '100%',
            objectFit: 'contain',
            zIndex: '-1',
            pointerEvents: 'none'
        });
    }
}

function updateRadarTransform() {
    if (radarContent) {
        radarContent.style.transform = `translate(${panX}px, ${panY}px) scale(${scale})`;
        radarContent.style.transformOrigin = '0 0';
    }
}

function smoothZoomOut() {
    isTrackingPlayer = false;
    focusedPlayer = null;

    const zoomOutSteps = 15;
    const targetScale = 1;
    const targetPanX = 0;
    const targetPanY = 0;

    const scaleStep = (targetScale - scale) / zoomOutSteps;
    const panXStep = (targetPanX - panX) / zoomOutSteps;
    const panYStep = (targetPanY - panY) / zoomOutSteps;

    let currentStep = 0;

    radarContent?.classList.add('no-transition');

    const zoomOutInterval = setInterval(() => {
        currentStep++;

        scale += scaleStep;
        panX += panXStep;
        panY += panYStep;

        updateRadarTransform();

        if (currentStep >= zoomOutSteps) {
            clearInterval(zoomOutInterval);
            scale = targetScale;
            panX = targetPanX;
            panY = targetPanY;

            radarContent?.classList.remove('no-transition');
            updateRadarTransform();
        }
    }, 40);
}

function stopTracking() {
    if (isTrackingPlayer) {
        smoothZoomOut();
    }
}

function calculatePlayerPosition(player) {
    const normalizedX = (player.position[0] - map_info.x) / map_info.scale;
    const normalizedY = (player.position[1] - map_info.y) / map_info.scale;
    const radarPixelX = normalizedX;
    const radarPixelY = -normalizedY;

    return {
        x: (radarPixelX / 1024) * 100,
        y: (radarPixelY / 1024) * 100
    };
}

function focusOnPlayer(player) {
    if (!player?.position?.length || !radar || !radarContent) return;

    if (isTrackingPlayer && focusedPlayer?.name === player.name) return;

    focusedPlayer = { ...player };
    isTrackingPlayer = true;

    const rect = radar.getBoundingClientRect();
    const radarWidth = rect.width;
    const radarHeight = rect.height;

    const position = calculatePlayerPosition(player);

    scale = trackingScale;

    const radarContentSize = Math.min(radarWidth, radarHeight);
    const playerActualX = (position.x / 100) * radarContentSize * scale;
    const playerActualY = (position.y / 100) * radarContentSize * scale;

    const centerX = radarWidth / 2;
    const centerY = radarHeight / 2;

    panX = centerX - playerActualX;
    panY = centerY - playerActualY;

    radarContent.classList.remove('no-transition');
    updateRadarTransform();

    checkTrackedPlayerHealth();
}

function checkTrackedPlayerHealth() {
    if (!isTrackingPlayer || !focusedPlayer) return;

    const allPlayers = [...(players || []), ...(friendlies || [])];
    const currentPlayerData = allPlayers.find(p => p.name === focusedPlayer.name);

    if (!currentPlayerData || currentPlayerData.health <= 0) {
        smoothZoomOut();
        return;
    }

    setTimeout(checkTrackedPlayerHealth, 100);
}

function addPlayers() {
    if (radarContent) {
        const existingPlayers = radarContent.querySelectorAll('.player-dot, .player-name, .bomb-carrier, .bomb-indicator, .bomb-timer');
        existingPlayers.forEach(el => el.remove());
    }

    for (const player of players || []) {
        addPlayer(player, false);
    }

    if (window.radarConfig.show_teammates) {
        for (const player of friendlies || []) {
            addPlayer(player, true);
        }
    }

    const allPlayers = [...(players || []), ...(friendlies || [])];
    for (const player of allPlayers) {
        if (player.has_bomb) {
            addBombCarrier(player);
            break;
        }
    }
}

function createPlayerElement(player, friendly) {
    const namespace = "http://www.w3.org/2000/svg";
    const svg = document.createElementNS(namespace, "svg");
    svg.setAttribute("xmlns", namespace);
    svg.setAttribute("class", "player-icon");
    svg.setAttribute("width", "20");
    svg.setAttribute("height", "20");
    svg.setAttribute("viewBox", "0 0 20 20");

    const circle = document.createElementNS(namespace, "circle");
    circle.setAttribute("cx", "10");
    circle.setAttribute("cy", "10");
    circle.setAttribute("r", "6");

    const arrow = document.createElementNS(namespace, "path");
    arrow.setAttribute("d", "M10 3 L14 9 L6 9 Z");

    svg.appendChild(circle);
    svg.appendChild(arrow);

    const rotation = (player && Number(player.rotation)) || 0;
    svg.style.transform = `rotate(${-rotation + 90}deg)`;

    let fillColor;
    if (friendly) {
        const config = window.radarConfig.teammate_dot_color;
        fillColor = `rgb(${config[0]}, ${config[1]}, ${config[2]})`;
    } else {
        if (window.radarConfig.enemy_dot_health_based) {
            fillColor = getHealthColor(player.health);
        } else {
            const config = window.radarConfig.enemy_dot_color;
            fillColor = `rgb(${config[0]}, ${config[1]}, ${config[2]})`;
        }
    }

    circle.setAttribute("fill", fillColor);
    circle.setAttribute("stroke", "#ffffff");
    circle.setAttribute("stroke-width", "2");
    arrow.setAttribute("fill", "#ffffff");
    arrow.setAttribute("stroke", fillColor);
    arrow.setAttribute("stroke-width", "1");

    return svg;
}

function addPlayerEventListeners(element, player) {
    const events = ['mousedown', 'click'];
    events.forEach(eventType => {
        element.addEventListener(eventType, (e) => {
            e.stopPropagation();
            e.preventDefault();
            focusOnPlayer(player);
        });
    });

    element.addEventListener('touchstart', (e) => {
        e.stopPropagation();
        element._touchStartTime = Date.now();
        element._touchMoved = false;
    });

    element.addEventListener('touchmove', (e) => {
        element._touchMoved = true;
    });

    element.addEventListener('touchend', (e) => {
        e.stopPropagation();
        e.preventDefault();

        const touchDuration = Date.now() - (element._touchStartTime || 0);
        if (!element._touchMoved && touchDuration < 200) {
            focusOnPlayer(player);
        }
    });
}

function addPlayer(player, friendly) {
    if (!player?.position || !radarContent) return;

    const playerDiv = document.createElement("div");
    playerDiv.className = friendly ? "player-dot teammate" : "player-dot enemy";

    if (friendly) {
        playerDiv.style.opacity = "0.6";
        playerDiv.style.filter = "brightness(0.8)";
    }

    addPlayerEventListeners(playerDiv, player);

    const svg = createPlayerElement(player, friendly);

    const position = calculatePlayerPosition(player);
    const svgOffset = 1.0;

    Object.assign(playerDiv.style, {
        left: (position.x - svgOffset) + "%",
                  top: (position.y - svgOffset) + "%",
                  position: 'absolute',
                  zIndex: '15',
                  cursor: 'pointer'
    });

    const healthClass = friendly ? "teammate" : getHealthClass(player.health);
    playerDiv.className = `player-dot ${healthClass}`;

    if (isTrackingPlayer && focusedPlayer?.name === player.name) {
        playerDiv.classList.add('focused');

        const isDead = player.health <= 0 || player.health === null || player.health === undefined;

        if (isDead) {
            smoothZoomOut();
        } else {
            const dx = Math.abs(player.position[0] - focusedPlayer.position[0]);
            const dy = Math.abs(player.position[1] - focusedPlayer.position[1]);

            if (dx > 50 || dy > 50) {
                focusedPlayer = { ...player };

                const rect = radar.getBoundingClientRect();
                const radarWidth = rect.width;
                const radarHeight = rect.height;

                const newPosition = calculatePlayerPosition(player);
                const radarContentSize = Math.min(radarWidth, radarHeight);
                const newPlayerActualX = (newPosition.x / 100) * radarContentSize * scale;
                const newPlayerActualY = (newPosition.y / 100) * radarContentSize * scale;

                const centerX = radarWidth / 2;
                const centerY = radarHeight / 2;

                panX = centerX - newPlayerActualX;
                panY = centerY - newPlayerActualY;

                updateRadarTransform();
            }
        }
    }

    playerDiv.appendChild(svg);
    radarContent.appendChild(playerDiv);

    const nameLabel = document.createElement("div");
    nameLabel.className = friendly ? "player-name teammate-name" : "player-name enemy-name";
    nameLabel.textContent = player.name || "Unknown";

    Object.assign(nameLabel.style, {
        left: (position.x + svgOffset) + "%",
                  top: (position.y - 3.5) + "%",
                  position: 'absolute',
                  zIndex: '20',
                  cursor: 'pointer'
    });

    if (friendly) {
        nameLabel.style.opacity = "0.7";
        nameLabel.style.color = "#5d9cec";
    }

    addPlayerEventListeners(nameLabel, player);

    nameLabel.addEventListener('mouseenter', () => {
        if (friendly) nameLabel.style.opacity = "1";
    });

        nameLabel.addEventListener('mouseleave', () => {
            if (friendly) nameLabel.style.opacity = "0.7";
        });

            radarContent.appendChild(nameLabel);
}

function addBombCarrier(player) {
    if (!radarContent || !player?.position) return;

    const bombCarrierIndicator = document.createElement("div");
    bombCarrierIndicator.className = "bomb-carrier";

    const position = calculatePlayerPosition(player);

    bombCarrierIndicator.textContent = "💣";
    Object.assign(bombCarrierIndicator.style, {
        left: (position.x + 0.5) + "%",
                  top: (position.y - 1.5) + "%",
                  position: 'absolute'
    });

    radarContent.appendChild(bombCarrierIndicator);
}

function getHealthColor(health) {
    const clampedHealth = Math.max(0, Math.min(100, health));

    if (clampedHealth >= 75) return "#00ff00";
    if (clampedHealth >= 50) return "#adff2f";
    if (clampedHealth >= 25) return "#ffa500";
    return "#ff4444";
}

function getHealthClass(health) {
    const clampedHealth = Math.max(0, Math.min(100, health));

    if (clampedHealth >= 75) return "health-high";
    if (clampedHealth >= 50) return "health-medium-high";
    if (clampedHealth >= 25) return "health-medium";
    return "health-low";
}

function updateRoundTimer() {
    const timerElement = document.getElementById("roundTime");
    if (timerElement) {
        const minutes = Math.floor(roundTime / 60);
        const seconds = roundTime % 60;
        timerElement.textContent = `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }
}

function updatePlayerLists() {
    const leftList = document.getElementById("leftPlayerList");
    const rightList = document.getElementById("rightPlayerList");

    if (leftList) leftList.innerHTML = "";
    if (rightList) rightList.innerHTML = "";

    const sortedEnemies = [...(players || [])].sort((a, b) => b.health - a.health);
    const sortedFriendlies = [...(friendlies || [])].sort((a, b) => b.health - a.health);

    sortedEnemies.forEach(player => {
        const card = createPlayerCard(player, 'terrorist');
        if (leftList) leftList.appendChild(card);
    });

        sortedFriendlies.forEach(player => {
            const card = createPlayerCard(player, 'counter-terrorist');
            if (rightList) rightList.appendChild(card);
        });
}

function createPlayerCard(player, team) {
    const card = document.createElement("div");
    card.className = `player-card ${team} ${player.health <= 0 ? 'dead' : ''}`;

    card.style.cursor = 'pointer';
    card.addEventListener('click', (e) => {
        e.stopPropagation();
        focusOnPlayer(player);
    });

    card.addEventListener('mouseenter', () => {
        card.style.transform = 'scale(1.02)';
        card.style.boxShadow = '0 4px 8px rgba(0,0,0,0.3)';
    });

    card.addEventListener('mouseleave', () => {
        card.style.transform = 'scale(1)';
        card.style.boxShadow = '';
    });

    const header = document.createElement("div");
    header.className = "player-header";

    const nameInfo = document.createElement("div");
    nameInfo.className = "player-name-info";

    const statusDot = document.createElement("div");
    statusDot.className = `player-status-dot ${player.health <= 0 ? 'dead' : ''}`;

    const nameText = document.createElement("span");
    nameText.textContent = player.name || "Unknown";

    nameInfo.appendChild(statusDot);
    nameInfo.appendChild(nameText);

    if (player.has_bomb) {
        const bombIcon = document.createElement("span");
        bombIcon.textContent = " 💣";
        bombIcon.style.fontSize = "14px";
        nameInfo.appendChild(bombIcon);
    }

    const money = document.createElement("div");
    money.className = "player-money";

    header.appendChild(nameInfo);
    header.appendChild(money);
    card.appendChild(header);

    const stats = document.createElement("div");
    stats.className = "player-stats";

    const healthBar = document.createElement("div");
    healthBar.className = "stat-bar";

    const healthFill = document.createElement("div");
    healthFill.className = "stat-fill health-fill";
    if (player.health <= 25) {
        healthFill.classList.add("low");
    } else if (player.health <= 50) {
        healthFill.classList.add("medium");
    }
    healthFill.style.width = `${Math.max(0, Math.min(100, player.health))}%`;

    const healthText = document.createElement("div");
    healthText.className = "stat-text";
    healthText.textContent = `${player.health} HP`;

    healthBar.appendChild(healthFill);
    healthBar.appendChild(healthText);

    const armorBar = document.createElement("div");
    armorBar.className = "stat-bar";

    const armor = player.health > 0 ? player.armor : 0;

    const armorFill = document.createElement("div");
    armorFill.className = "stat-fill armor-fill";
    armorFill.style.width = `${Math.max(0, Math.min(100, armor))}%`;

    const armorText = document.createElement("div");
    armorText.className = "stat-text";
    armorText.textContent = `${armor} AR`;

    armorBar.appendChild(armorFill);
    armorBar.appendChild(armorText);

    stats.appendChild(healthBar);
    stats.appendChild(armorBar);
    card.appendChild(stats);

    const equipment = document.createElement("div");
    equipment.className = "player-equipment";

    if (player.weapon) {
        const weaponSlot = document.createElement("div");
        weaponSlot.className = "weapon-slot active";
        weaponSlot.textContent = player.weapon;
        equipment.appendChild(weaponSlot);
    }

    if (player.has_defuser) {
        const defuserIcon = document.createElement("div");
        defuserIcon.className = "equipment-icon has-item";
        defuserIcon.textContent = "🔧";
        equipment.appendChild(defuserIcon);
    }

    if (player.has_helmet) {
        const helmetIcon = document.createElement("div");
        helmetIcon.className = "equipment-icon has-item";
        helmetIcon.textContent = "🛡️";
        equipment.appendChild(helmetIcon);
    }

    card.appendChild(equipment);

    return card;
}

setInterval(() => {
    if (roundTime > 0) {
        roundTime--;
        updateRoundTimer();
    }
}, 1000);

setInterval(() => {
    if (ws?.readyState !== 1) {
        startWebSocket();
    }
}, 5000);

setInterval(() => {
    if (ws?.readyState === 1) {
        const message = JSON.stringify({ kind: "get_data", uuid: uuid });
        ws.send(message);
    }
}, 50);

startWebSocket();

/**
 * @typedef {Object} MapInfo
 * @property {number} x
 * @property {number} y
 * @property {number} scale
 * @property {boolean} rotate
 * @property {number} zoom
 * @property {number?} lowerThreshold
 */

/**
 * @typedef {string} Bones
 */

/**
 * @typedef {Object} BombData
 * @property {boolean} planted
 * @property {number} timer
 * @property {boolean} being_defused
 * @property {Array<number>} position
 */

/**
 * @typedef {Object} PlayerData
 * @property {number} health
 * @property {number} armor
 * @property {Array<number>} position
 * @property {Array<number>} head
 * @property {string} name
 * @property {string} weapon
 * @property {Record<Bones, Array<number>>} bones
 * @property {boolean} has_defuser
 * @property {boolean} has_helmet
 * @property {boolean} has_bomb
 * @property {boolean} visible
 * @property {number} color
 * @property {number} rotation
 */

/**
 * @typedef {Object} WeaponPosition
 * @property {Weapon} weapon
 * @property {Vec3} position
 */

/**
 * @typedef {Object} Data
 * @property {boolean} in_game
 * @property {boolean} is_ffa
 * @property {Weapon} weapon
 * @property {PlayerData[]} players
 * @property {PlayerData[]} friendlies
 * @property {PlayerData} local_player
 * @property {WeaponPosition[]} weapons
 * @property {BombData} bomb
 * @property {string} map_name
 * @property {Array<number>} window_position
 * @property {Array<number>} window_size
 * @property {boolean} triggerbot_active
 */
