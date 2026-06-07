const express = require("express");
const chokidar = require("chokidar");
const fs = require("fs");
const path = require("path");

const viewerRoot = __dirname;
const projectRoot = path.resolve(viewerRoot, "..", "..");
const distRoot = path.join(viewerRoot, "dist");
const assetsRoot = path.join(projectRoot, "assets");
const levelsRoot = path.join(assetsRoot, "levels");

const host = process.env.HOST || "127.0.0.1";
const port = Number(process.env.PORT || 0);
const strictPort = process.env.STRICT_PORT === "true";

const pastoralSprites = {
  grass_plain: { col: 6, row: 0 },
  grass_tuft_left: { col: 8, row: 0 },
  grass_tuft_right: { col: 9, row: 0 },
  grass_tuft_lower: { col: 9, row: 1 },
  open_water: { col: 6, row: 2 },
  tree_stump: { col: 0, row: 10 },
};
const pastoralSpriteNamesByCoordinate = new Map(
  Object.entries(pastoralSprites).map(([name, coordinates]) => [
    `${coordinates.col}:${coordinates.row}`,
    name,
  ]),
);

const app = express();
const sseClients = new Set();

app.use(express.json({ limit: "2mb" }));

function ensureLevelsDirectory() {
  fs.mkdirSync(levelsRoot, { recursive: true });
}

function listLevelNames() {
  ensureLevelsDirectory();

  return fs
    .readdirSync(levelsRoot, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".json"))
    .map((entry) => path.basename(entry.name, ".json"))
    .sort((left, right) => left.localeCompare(right));
}

function sanitizeLevelName(levelName) {
  if (!/^[a-zA-Z0-9_-]+$/.test(levelName)) {
    throw new Error(`Invalid level name '${levelName}'`);
  }

  return levelName;
}

function resolvePastoralSprite(sprite) {
  if (sprite.name) {
    const resolved = pastoralSprites[sprite.name];
    if (!resolved) {
      throw new Error(`Unknown pastoral sprite '${sprite.name}'`);
    }

    return resolved;
  }

  if (Number.isInteger(sprite.col) && Number.isInteger(sprite.row)) {
    return { col: sprite.col, row: sprite.row };
  }

  throw new Error("Pastoral sprite requires either 'name' or both 'col' and 'row'");
}

function normalizeSprite(sprite) {
  if (!sprite || typeof sprite !== "object") {
    throw new Error("Tile sprite must be an object");
  }

  switch (sprite.tileset) {
    case "pastoral": {
      const resolved = resolvePastoralSprite(sprite);
      return {
        tileset: "pastoral",
        col: resolved.col,
        row: resolved.row,
      };
    }
    default:
      throw new Error(`Unsupported tileset '${sprite.tileset}'`);
  }
}

function normalizeTileEffect(effect, rowIndex, colIndex, effectIndex) {
  if (!effect || typeof effect !== "object") {
    throw new Error(
      `Effect ${effectIndex} at row ${rowIndex}, col ${colIndex} must be an object`,
    );
  }

  const tileEffectType = effect.tile_effect_type ?? effect.tileEffectType;
  if (typeof tileEffectType !== "string" || tileEffectType.trim().length === 0) {
    throw new Error(
      `Effect ${effectIndex} at row ${rowIndex}, col ${colIndex} requires tile_effect_type`,
    );
  }

  const modifier =
    typeof effect.modifier === "number" && Number.isFinite(effect.modifier)
      ? effect.modifier
      : 0;
  const extraData =
    typeof effect.extra_data === "string"
      ? effect.extra_data
      : typeof effect.extraData === "string"
        ? effect.extraData
        : "";

  return {
    tileEffectType: tileEffectType.trim(),
    modifier,
    extraData: extraData.trim().length > 0 ? extraData.trim() : null,
  };
}

function normalizeTile(tile, rowIndex, colIndex) {
  if (!tile || typeof tile !== "object") {
    throw new Error(`Tile at row ${rowIndex}, col ${colIndex} must be an object`);
  }

  if (tile.effects !== undefined && !Array.isArray(tile.effects)) {
    throw new Error(`Tile effects at row ${rowIndex}, col ${colIndex} must be an array`);
  }

  return {
    sprite: normalizeSprite(tile.sprite),
    walkable: tile.walkable ?? true,
    zLayer:
      typeof tile.z_layer === "number"
        ? tile.z_layer
        : typeof tile.zLayer === "number"
          ? tile.zLayer
          : 0,
    effects: (tile.effects ?? []).map((effect, effectIndex) =>
      normalizeTileEffect(effect, rowIndex, colIndex, effectIndex),
    ),
    tag: typeof tile.tag === "string" ? tile.tag : null,
  };
}

function normalizeLevel(levelName, payload) {
  if (!payload || typeof payload !== "object") {
    throw new Error("Level JSON must be an object");
  }

  const playerSpawn = payload.player_spawn ?? payload.playerSpawn;
  const tiles = payload.tiles;
  if (!playerSpawn || !Number.isInteger(playerSpawn.x) || !Number.isInteger(playerSpawn.y)) {
    throw new Error("player_spawn must contain integer x and y");
  }

  if (!Array.isArray(tiles) || tiles.length === 0) {
    throw new Error("tiles must be a non-empty 2D array");
  }

  const width = Array.isArray(tiles[0]) ? tiles[0].length : 0;
  if (width === 0) {
    throw new Error("tiles rows must not be empty");
  }

  const normalizedTiles = tiles.map((row, rowIndex) => {
    if (!Array.isArray(row)) {
      throw new Error(`Row ${rowIndex} must be an array`);
    }

    if (row.length !== width) {
      throw new Error(`Row ${rowIndex} has width ${row.length}, expected ${width}`);
    }

    return row.map((tile, colIndex) => normalizeTile(tile, rowIndex, colIndex));
  });

  return {
    name: levelName,
    width,
    height: normalizedTiles.length,
    playerSpawn: {
      x: playerSpawn.x,
      y: playerSpawn.y,
    },
    tiles: normalizedTiles,
  };
}

function readLevel(levelName) {
  const safeName = sanitizeLevelName(levelName);
  const filePath = path.join(levelsRoot, `${safeName}.json`);

  if (!fs.existsSync(filePath)) {
    const error = new Error(`Level '${safeName}' not found`);
    error.statusCode = 404;
    throw error;
  }

  const rawContents = fs.readFileSync(filePath, "utf8");
  const parsed = JSON.parse(rawContents);
  return normalizeLevel(safeName, parsed);
}

function collectLevelErrors(levelNames) {
  const errors = [];

  for (const levelName of levelNames) {
    try {
      readLevel(levelName);
    } catch (error) {
      errors.push({
        level: levelName,
        message: error.message,
      });
    }
  }

  return errors;
}

function serializeSprite(sprite) {
  if (sprite.tileset === "pastoral") {
    const spriteName = pastoralSpriteNamesByCoordinate.get(`${sprite.col}:${sprite.row}`);
    if (spriteName) {
      return {
        tileset: "pastoral",
        name: spriteName,
      };
    }
  }

  return {
    tileset: sprite.tileset,
    col: sprite.col,
    row: sprite.row,
  };
}

function serializeTileEffect(effect) {
  const payload = {
    tile_effect_type: effect.tileEffectType,
    modifier: effect.modifier,
  };

  const extraData = typeof effect.extraData === "string" ? effect.extraData.trim() : "";
  if (extraData.length > 0) {
    payload.extra_data = extraData;
  }

  return payload;
}

function serializeTile(tile) {
  const payload = {
    sprite: serializeSprite(tile.sprite),
  };

  if (tile.walkable === false) {
    payload.walkable = false;
  }

  if (tile.zLayer !== 0) {
    payload.z_layer = tile.zLayer;
  }

  if (Array.isArray(tile.effects) && tile.effects.length > 0) {
    payload.effects = tile.effects.map(serializeTileEffect);
  }

  const tag = typeof tile.tag === "string" ? tile.tag.trim() : "";
  if (tag.length > 0) {
    payload.tag = tag;
  }

  return payload;
}

function serializeLevel(level) {
  return {
    player_spawn: {
      x: level.playerSpawn.x,
      y: level.playerSpawn.y,
    },
    tiles: level.tiles.map((row) => row.map(serializeTile)),
  };
}

function broadcastReload(payload) {
  const message = `event: reload\ndata: ${JSON.stringify(payload)}\n\n`;

  for (const client of sseClients) {
    client.write(message);
  }
}

app.get("/api/levels", (_request, response) => {
  const levels = listLevelNames();
  response.setHeader("Cache-Control", "no-store");
  response.json({
    levels,
    errors: collectLevelErrors(levels),
  });
});

app.get("/api/levels/:levelName", (request, response) => {
  try {
    const level = readLevel(request.params.levelName);
    response.setHeader("Cache-Control", "no-store");
    response.json(level);
  } catch (error) {
    response.status(error.statusCode || 400).json({
      error: error.message,
    });
  }
});

app.put("/api/levels/:levelName", (request, response) => {
  try {
    const levelName = sanitizeLevelName(request.params.levelName);
    const normalizedLevel = normalizeLevel(levelName, request.body);
    const filePath = path.join(levelsRoot, `${levelName}.json`);
    const serializedLevel = serializeLevel(normalizedLevel);

    fs.writeFileSync(filePath, `${JSON.stringify(serializedLevel, null, 2)}\n`, "utf8");

    response.setHeader("Cache-Control", "no-store");
    response.json({
      level: normalizedLevel,
      savedPath: path.relative(projectRoot, filePath).replace(/\\/g, "/"),
    });
  } catch (error) {
    response.status(error.statusCode || 400).json({
      error: error.message,
    });
  }
});

app.get("/events", (request, response) => {
  response.setHeader("Content-Type", "text/event-stream");
  response.setHeader("Cache-Control", "no-cache, no-transform");
  response.setHeader("Connection", "keep-alive");
  response.flushHeaders();

  response.write(`event: ready\ndata: {}\n\n`);
  sseClients.add(response);

  const keepAlive = setInterval(() => {
    response.write(": keep-alive\n\n");
  }, 15000);

  request.on("close", () => {
    clearInterval(keepAlive);
    sseClients.delete(response);
  });
});

app.use(
  "/game-assets",
  express.static(assetsRoot, {
    etag: false,
    index: false,
    setHeaders: (response) => {
      response.setHeader("Cache-Control", "no-store");
    },
  }),
);

if (fs.existsSync(distRoot)) {
  app.use(
    express.static(distRoot, {
      etag: false,
      setHeaders: (response) => {
        response.setHeader("Cache-Control", "no-store");
      },
    }),
  );

  app.get("*", (_request, response) => {
    response.sendFile(path.join(distRoot, "index.html"));
  });
} else {
  app.get("*", (_request, response) => {
    response.status(503).send(
      [
        "Level viewer frontend not built.",
        "Use `npm run dev` for development or `npm run build` before `npm start`.",
      ].join(" "),
    );
  });
}

ensureLevelsDirectory();

const watcher = chokidar.watch([levelsRoot, path.join(assetsRoot, "pastoral-tileset.png")], {
  ignoreInitial: true,
});

watcher.on("all", (eventName, filePath) => {
  broadcastReload({
    eventName,
    filePath: path.relative(projectRoot, filePath).replace(/\\/g, "/"),
    timestamp: Date.now(),
  });
});

function startServer(currentPort) {
  const server = app.listen(currentPort, host, () => {
    const address = server.address();
    const resolvedPort =
      address && typeof address !== "string" ? address.port : currentPort;

    console.log(`Level viewer listening at http://${host}:${resolvedPort}`);
  });

  server.on("error", (error) => {
    if (error.code === "EADDRINUSE") {
      if (strictPort) {
        throw error;
      }

      const nextPort = currentPort + 1;
      console.warn(`Port ${currentPort} is busy, retrying on ${nextPort}...`);
      startServer(nextPort);
      return;
    }

    throw error;
  });
}

startServer(port);
