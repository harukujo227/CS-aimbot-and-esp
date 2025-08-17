import express from "express";
import { WebSocket, WebSocketServer } from "ws";
import path from "path";
import http from "http";
import { v4 } from "uuid";

const app = express();
const PORT = 6346;

app.use(express.static(path.join(__dirname, "public")));

const server = http.createServer(app);

const ws = new WebSocketServer({ server, path: "/" });

const games: Record<string, { data: any; last_update: number }> = {};
ws.on("connection", (ws, req) => {
    console.info(`connection from ${req.socket.remoteAddress}`);

    ws.on("message", (message) => {
        const data = JSON.parse(message.toString());

        const kind: string = data["kind"];
        if (kind == "connect_server") {
            connect_server(ws);
        } else if (kind == "update_data") {
            update_data(data);
        } else if (kind == "get_data") {
            get_data(ws, data);
        }
    });

    ws.on("error", (error) => {
        console.error(`error: ${error}`);
    });
});

function connect_server(ws: WebSocket) {
    const uuid = v4();
    games[uuid] = { data: {}, last_update: Date.now() };
    const message = { kind: "uuid", uuid: uuid };
    ws.send(JSON.stringify(message));
}

function update_data(data: any) {
    const uuid = data["uuid"];
    if (uuid in games) {
        games[uuid].data = data["data"];
        games[uuid].last_update = Date.now();
    }
}

function get_data(ws: WebSocket, data: any) {
    const uuid = data["uuid"];
    if (uuid in games) {
        const message = JSON.stringify(games[uuid].data);
        ws.send(message);
    }
}

setInterval(() => {
    const now = Date.now();
    for (const [uuid, game] of Object.entries(games)) {
        if (now - game.last_update > 1_200_000) {
            delete games[uuid];
        }
    }
}, 5000);

server.listen(PORT, () => {
    console.info(`listening on port ${PORT}`);
});
