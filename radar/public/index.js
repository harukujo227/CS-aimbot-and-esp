/** @type{WebSocket | null} */
let ws = null;

/** @type {Record<string, MapInfo>} */
const MapData = {
    de_ancient_night: { x: -2953, y: 2164, scale: 5, rotate: false, zoom: 1 },
    de_dust2: { x: -2476, y: 3239, scale: 4.4, rotate: true, zoom: 1.1 },
    de_inferno: { x: -2087, y: 3870, scale: 4.9, rotate: false, zoom: 1 },
    de_mirage: { x: -3230, y: 1713, scale: 5, rotate: false, zoom: 1 },
    de_nuke: { x: -3453, y: 2887, scale: 7, rotate: false, zoom: 1, lowerThreshold: -495 },
    de_overpass: { x: -4831, y: 1781, scale: 5.2, rotate: false, zoom: 1 },
    de_vertigo: { x: -3168, y: 1762, scale: 4, rotate: false, zoom: 1, lowerThreshold: 11700 },
};

const loc = window.location;
const uuid = new URLSearchParams(loc.search).get("uuid");
let url = `ws://${loc.hostname}:${loc.port}`;
/** @type {PlayerData[]} */
let players = [];
/** @type {PlayerData[]} */
let friendlies = [];
/** @type {BombData} */
let bombData = null;
/** @type {HTMLDivElement} */
const radar = document.getElementById("radar");
/** @type {MapInfo} */
let map_info = MapData["de_mirage"];
let currentMapName = "de_mirage";

// Set initial map background
radar.style.backgroundImage = `url("/radars/de_mirage.png")`;

function startWebSocket() {
    stopWebSocket();
    ws = new WebSocket(url);
    ws.onmessage = wsMessage;
}

function stopWebSocket() {
    ws?.close();
    ws = null;
}

/** @param {MessageEvent} event  */
function wsMessage(event) {
    /** @type {Data} */
    const json = JSON.parse(event.data);
    players = json.players;
    friendlies = json.friendlies;
    bombData = json.bomb;

    // Update map
    if (json.map_name && json.map_name !== currentMapName) {
        currentMapName = json.map_name;
        radar.style.backgroundImage = `url("/radars/${json.map_name}.png")`;
        map_info = MapData[json.map_name];

        // Update map name display
        const mapNameElement = document.getElementById("mapName");
        if (mapNameElement) {
            mapNameElement.textContent = json.map_name.replace("de_", "").toUpperCase();
            console.log("Incoming map_name:", json.map_name);

        }
    }

    addPlayers();
    updatePlayerLists();
}

function addPlayers() {
    // Clear only player elements, keep overlay
    const existingPlayers = radar.querySelectorAll('.player-dot, .player-name, .bomb-carrier');
    existingPlayers.forEach(el => el.remove());

    // Only show enemies (players array), not friendlies
    for (const player of players || []) {
        addPlayer(player, false);
    }

    // Show bomb carrier (can be enemy or friendly)
    const allPlayers = [...(players || []), ...(friendlies || [])];
    for (const player of allPlayers) {
        if (player.has_bomb) {
            addBombCarrier(player);
            break; // Only one player can have bomb
        }
    }
}

function addPlayer(player, friendly) {
    const namespace = "http://www.w3.org/2000/svg";
    const svg = document.createElementNS(namespace, "svg");
    svg.setAttribute("xmlns", namespace);
    svg.setAttribute("class", "player-icon");
    svg.setAttribute("width", "20");
    svg.setAttribute("height", "20");
    svg.setAttribute("viewBox", "0 0 20 20");

    const playerDiv = document.createElement("div");
    playerDiv.className = "player-dot";

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

    const radarSize = 1024;
    const normalizedX = (player.position[0] - map_info.x) / map_info.scale;
    const normalizedY = (player.position[1] - map_info.y) / map_info.scale;
    const radarPixelX = normalizedX;
    const radarPixelY = -normalizedY;

    const svgOffset = 1.0;
    const position = {
        x: (radarPixelX / 1024) * 100 - svgOffset,
        y: (radarPixelY / 1024) * 100 - svgOffset
    };

    playerDiv.style.left = position.x + "%";
    playerDiv.style.top = position.y + "%";

    const fill = getHealthColor(player.health);
    circle.setAttribute("fill", fill);
    circle.setAttribute("stroke", "#ffffff");
    arrow.setAttribute("fill", "#ffffff");
    arrow.setAttribute("stroke", fill);

    playerDiv.appendChild(svg);
    radar.appendChild(playerDiv);

    const nameLabel = document.createElement("div");
    nameLabel.className = "player-name";
    nameLabel.textContent = player.name || "Unknown";
    nameLabel.style.left = (position.x + svgOffset) + "%";
    nameLabel.style.top = (position.y - 2) + "%";

    radar.appendChild(nameLabel);
}

function addBombCarrier(player) {
    const bombCarrierIndicator = document.createElement("div");
    bombCarrierIndicator.className = "bomb-carrier";

    const normalizedX = (player.position[0] - map_info.x) / map_info.scale;
    const normalizedY = (player.position[1] - map_info.y) / map_info.scale;
    const radarPixelX = normalizedX;
    const radarPixelY = -normalizedY;

    const position = {
        x: (radarPixelX / 1024) * 100 + 0.5,
        y: (radarPixelY / 1024) * 100 - 1.5
    };

    bombCarrierIndicator.textContent = "💣";
    bombCarrierIndicator.style.left = position.x + "%";
    bombCarrierIndicator.style.top = position.y + "%";

    radar.appendChild(bombCarrierIndicator);
}

function getHealthColor(health) {
    const clamped = Math.max(0, Math.min(100, health));
    if (clamped >= 75) return "#00ff00";
    if (clamped >= 50) return "#adff2f";
    if (clamped >= 25) return "#ffa500";
    return "#ff4444";
}

function getHealthClass(health) {
    const clampedHealth = Math.max(0, Math.min(100, health));

    if (clampedHealth >= 75) {
        return "health-high";
    } else if (clampedHealth >= 50) {
        return "health-medium-high";
    } else if (clampedHealth >= 25) {
        return "health-medium";
    } else {
        return "health-low";
    }
}

// Update round timer
function updateRoundTimer() {
    const timerElement = document.getElementById("roundTime");
    if (timerElement) {
        // This would normally come from your WebSocket data
        // For now, just showing a placeholder
        const minutes = Math.floor(115 / 60);
        const seconds = 115 % 60;
        timerElement.textContent = `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }
}

// Connection management
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

// Start the connection
startWebSocket();

// Update player lists in sidebars
function updatePlayerLists() {
    const leftList = document.getElementById("leftPlayerList");   // CT side
    const rightList = document.getElementById("rightPlayerList"); // T side

    leftList.innerHTML = "";
    rightList.innerHTML = "";

    // Sort enemies and friendlies by health (alive first)
    const sortedEnemies = [...(players || [])].sort((a, b) => b.health - a.health);
    const sortedFriendlies = [...(friendlies || [])].sort((a, b) => b.health - a.health);

    // Enemies on left
    sortedEnemies.forEach(player => {
        const card = createPlayerCard(player, 'enemy'); // enemy style
        leftList.appendChild(card);
    });

    // Friendlies on right
    sortedFriendlies.forEach(player => {
        const card = createPlayerCard(player, 'friendly'); // friendly style
        rightList.appendChild(card);
    });
}

// Create player card element
function createPlayerCard(player, team) {
    const card = document.createElement("div");
    card.className = `player-card ${team} ${player.health <= 0 ? 'dead' : ''}`;

    // Header with name and money
    const header = document.createElement("div");
    header.className = "player-header";

    const nameInfo = document.createElement("div");
    nameInfo.className = "player-name-info";

    const statusIcon = document.createElement("span");
    statusIcon.className = `player-status-icon ${player.health <= 0 ? 'dead' : ''}`;

    const nameText = document.createElement("span");
    nameText.textContent = player.name || "Unknown";

    nameInfo.appendChild(statusIcon);
    nameInfo.appendChild(nameText);

    // Add bomb indicator if player has bomb
    if (player.has_bomb && !bombData?.planted) {
        const bombIcon = document.createElement("span");
        bombIcon.textContent = " 💣";
        bombIcon.style.fontSize = "14px";
        nameInfo.appendChild(bombIcon);
    }

    const money = document.createElement("div");
    money.className = "player-money";
 //   money.textContent = `${(Math.random() * 16000).toFixed(0)}`;

    header.appendChild(nameInfo);
    header.appendChild(money);
    card.appendChild(header);

    // Stats bars (health and armor)
    const stats = document.createElement("div");
    stats.className = "player-stats";

    // Health bar
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

    // Armor bar
    const armorBar = document.createElement("div");
    armorBar.className = "stat-bar";

    const armorFill = document.createElement("div");
    armorFill.className = "stat-fill armor-fill";
    armorFill.style.width = `${Math.max(0, Math.min(100, player.armor))}%`;

    const armorText = document.createElement("div");
    armorText.className = "stat-text";
    armorText.textContent = `${player.armor} AR`;

    armorBar.appendChild(armorFill);
    armorBar.appendChild(armorText);

    stats.appendChild(healthBar);
    stats.appendChild(armorBar);
    card.appendChild(stats);

    // Equipment section
    const equipment = document.createElement("div");
    equipment.className = "player-equipment";

    // Weapon
    if (player.weapon) {
        const weaponSlot = document.createElement("div");
        weaponSlot.className = "weapon-slot primary active";
        weaponSlot.textContent = player.weapon;
        equipment.appendChild(weaponSlot);
    }

    // Utility icons
    const utilities = [
        { icon: player.has_defuser ? "🔧" : "", class: player.has_defuser ? "has-item" : "" },
        { icon: player.has_helmet ? "🛡️" : "", class: player.has_helmet ? "has-item" : "" }
    ];

    utilities.forEach(util => {
        if (util.icon) {
            const utilIcon = document.createElement("div");
            utilIcon.className = `equipment-icon ${util.class}`;
            utilIcon.textContent = util.icon;
            equipment.appendChild(utilIcon);
        }
    });

    card.appendChild(equipment);

    return card;
}

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
