/** @type{WebSocket | null} */
let ws = null;

/** @type {Record<string, MapInfo>} */
const MapData = {
    de_ancient: {
        x: -2953,
        y: 2164,
        scale: 5,
        rotate: false,
        zoom: 1,
    },
    de_dust2: {
        x: -2476,
        y: 3239,
        scale: 4.4,
        rotate: true,
        zoom: 1.1,
    },
    de_inferno: {
        x: -2087,
        y: 3870,
        scale: 4.9,
        rotate: false,
        zoom: 1,
    },
    de_mirage: {
        x: -3230,
        y: 1713,
        scale: 5,
        rotate: false,
        zoom: 1,
    },
    de_nuke: {
        x: -3453,
        y: 2887,
        scale: 7,
        rotate: false,
        zoom: 1,
        lowerThreshold: -495,
    },
    de_overpass: {
        x: -4831,
        y: 1781,
        scale: 5.2,
        rotate: false,
        zoom: 1,
    },
    de_vertigo: {
        x: -3168,
        y: 1762,
        scale: 4,
        rotate: false,
        zoom: 1,
        lowerThreshold: 11700,
    },
};

const loc = window.location;
const uuid = new URLSearchParams(loc.search).get("uuid");
let url = `ws://${loc.hostname}:${loc.port}`;
/** @type {PlayerData[]} */
let players = [];
/** @type {PlayerData[]} */
let friendlies = [];
/** @type {HTMLDivElement} */
const radar = document.getElementById("radar");
/** @type {MapInfo} */
let map_info = MapData["de_dust2"];

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
    radar.style.backgroundImage = `url("/radars/${json.map_name}.png")`;
    map_info = MapData[json.map_name];

    addPlayers();
}

function addPlayers() {
    radar.innerHTML = "";
    for (const player of players) {
        addPlayer(player, false);
    }
    for (const player of friendlies) {
        addPlayer(player, true);
    }
}

/** @param {PlayerData} player @param {boolean} friendly */
function addPlayer(player, friendly) {
    const namespace = "http://www.w3.org/2000/svg";
    const svg = document.createElementNS(namespace, "svg");
    svg.setAttribute("xmlns", namespace);
    svg.setAttribute("width", "24");
    svg.setAttribute("height", "24");
    svg.setAttribute("viewBox", "0 0 24 24");

    const style = svg.style;
    style.position = "absolute";
    style.width = "2.5%";
    style.height = "2.5%";
    style.transition = "transform 0.02s, left 0.02s, top 0.02s, opacity 0.02s";

    const path = document.createElementNS(namespace, "path");
    path.setAttribute(
        "d",
        "M10.708 2.372a2.382 2.382 0 0 0 -.71 .686l-4.892 7.26c-1.981 3.314 -1.22 7.466 1.767 9.882c2.969 2.402 7.286 2.402 10.254 0c2.987 -2.416 3.748 -6.569 1.795 -9.836l-4.919 -7.306c-.722 -1.075 -2.192 -1.376 -3.295 -.686z"
    );

    svg.appendChild(path);

    const rotation = 90 - ((player && Number(player.rotation)) || 0);
    svg.style.transform = `rotate(${rotation}deg)`;

    // position: left: position.x px; top: -position.y px
    const position = {
        x: ((player.position[0] - map_info.x) / map_info.scale) * (radar.clientWidth / 1024),
        y: ((player.position[1] - map_info.y) / map_info.scale) * (radar.clientHeight / 1024),
    };
    const px = (n) => (Number(n) || 0) + "px";
    svg.style.left = px(position.x);
    svg.style.top = px(-position.y);

    // todo: opacity: 1 or 0.5
    svg.style.opacity = "1";

    // fill: friendly ? getColor(player.color) : var(--color-red)
    const fill = friendly ? getColor(player.color) : "red";
    path.setAttribute("fill", fill);

    radar.appendChild(svg);
}

const colors = ["blue", "green", "yellow", "orange", "purple", "white"];

/** @param {number} index */
function getColor(index) {
    const color = colors[index];
    if (color === undefined) {
        return colors[5];
    }
    return color;
}

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

/**
 * @typedef {Object} MapInfo
 *  @property {number}x
 *  @property {number} y
 *  @property {number} scale
 *  @property {boolean} rotate
 *  @property {number} zoom
 *  @property {number?} lowerThreshold
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
