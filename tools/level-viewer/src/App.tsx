import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";

type TilesetName = "pastoral";

const TILE_EFFECT_TYPES = [
  "damage",
  "heal",
  "slow",
  "speed",
  "teleport",
  "voyage",
  "message",
  "blockage",
] as const;

type TileEffectType = (typeof TILE_EFFECT_TYPES)[number];

type LevelTileEffect = {
  tileEffectType: TileEffectType;
  modifier: number;
  extraData: string | null;
};

type LevelApiSummary = {
  levels: string[];
  errors: Array<{
    level: string;
    message: string;
  }>;
};

type LevelTile = {
  sprite: {
    tileset: TilesetName;
    col: number;
    row: number;
  };
  walkable: boolean;
  zLayer: number;
  effects: LevelTileEffect[];
  tag: string | null;
};

type LevelData = {
  name: string;
  width: number;
  height: number;
  playerSpawn: {
    x: number;
    y: number;
  };
  tiles: LevelTile[][];
};

type SaveLevelResponse = {
  level: LevelData;
  savedPath: string;
};

type ViewOffset = {
  x: number;
  y: number;
};

type ViewportSize = {
  width: number;
  height: number;
};

type GridPosition = {
  col: number;
  row: number;
};

type Brush = {
  sprite: LevelTile["sprite"];
  walkable: boolean;
  zLayer: number;
  effects: LevelTileEffect[];
  tag: string;
};

type ReloadEventPayload = {
  eventName: string;
  filePath: string;
  timestamp: number;
};

type InteractionState =
  | {
      mode: "idle";
    }
  | {
      mode: "paint";
      pointerId: number;
      lastPaintKey: string | null;
    }
  | {
      mode: "pan";
      pointerId: number;
      lastClientX: number;
      lastClientY: number;
    };

const TILESETS: Record<
  TilesetName,
  { src: string; tileSize: number; columns: number; rows: number }
> = {
  pastoral: {
    src: "/game-assets/pastoral-tileset.png",
    tileSize: 16,
    columns: 12,
    rows: 16,
  },
};

const DEFAULT_TILE_META = "Hover a tile to inspect it.";
const PALETTE_SCALE = 2;
const DEFAULT_BRUSH: Brush = {
  sprite: {
    tileset: "pastoral",
    col: 6,
    row: 0,
  },
  walkable: true,
  zLayer: 0,
  effects: [],
  tag: "",
};

function useLevelImages(cacheBust: number) {
  const imageCache = useRef(new Map<string, HTMLImageElement>());

  useEffect(() => {
    imageCache.current.clear();
  }, [cacheBust]);

  return async (tilesetName: TilesetName) => {
    const cacheKey = `${tilesetName}:${cacheBust}`;
    const cachedImage = imageCache.current.get(cacheKey);
    if (cachedImage) {
      return cachedImage;
    }

    const image = new Image();
    image.src = `${TILESETS[tilesetName].src}?v=${cacheBust}`;
    await image.decode();
    imageCache.current.set(cacheKey, image);
    return image;
  };
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url, { cache: "no-store" });
  const payload = await response.json();

  if (!response.ok) {
    throw new Error(payload.error || `Request failed for ${url}`);
  }

  return payload as T;
}

async function sendJson<T>(url: string, method: "PUT", body: unknown): Promise<T> {
  const response = await fetch(url, {
    method,
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  const payload = await response.json();

  if (!response.ok) {
    throw new Error(payload.error || `Request failed for ${url}`);
  }

  return payload as T;
}

function createDefaultTile(): LevelTile {
  return {
    sprite: { ...DEFAULT_BRUSH.sprite },
    walkable: true,
    zLayer: 0,
    effects: [],
    tag: null,
  };
}

function createDefaultEffect(): LevelTileEffect {
  return {
    tileEffectType: "blockage",
    modifier: 0,
    extraData: null,
  };
}

function cloneEffect(effect: LevelTileEffect): LevelTileEffect {
  return {
    tileEffectType: effect.tileEffectType,
    modifier: effect.modifier,
    extraData: effect.extraData,
  };
}

function cloneTile(tile: LevelTile): LevelTile {
  return {
    sprite: { ...tile.sprite },
    walkable: tile.walkable,
    zLayer: tile.zLayer,
    effects: tile.effects.map(cloneEffect),
    tag: tile.tag,
  };
}

function cloneLevel(level: LevelData): LevelData {
  return {
    name: level.name,
    width: level.width,
    height: level.height,
    playerSpawn: { ...level.playerSpawn },
    tiles: level.tiles.map((row) => row.map(cloneTile)),
  };
}

function synchronizeLevelDimensions(level: LevelData) {
  level.height = level.tiles.length;
  level.width = level.tiles[0]?.length ?? 0;
}

function applyBrushToLevel(
  level: LevelData,
  gridPosition: GridPosition,
  brush: Brush,
): LevelData {
  const nextLevel = cloneLevel(level);
  let { col, row } = gridPosition;

  if (row < 0) {
    const rowsToPrepend = Math.abs(row);
    const newRows = Array.from({ length: rowsToPrepend }, () =>
      Array.from({ length: nextLevel.width }, createDefaultTile),
    );
    nextLevel.tiles = [...newRows, ...nextLevel.tiles];
    nextLevel.playerSpawn.y += rowsToPrepend;
    row = 0;
  }

  if (col < 0) {
    const columnsToPrepend = Math.abs(col);
    nextLevel.tiles = nextLevel.tiles.map((existingRow) => [
      ...Array.from({ length: columnsToPrepend }, createDefaultTile),
      ...existingRow,
    ]);
    nextLevel.playerSpawn.x += columnsToPrepend;
    col = 0;
  }

  synchronizeLevelDimensions(nextLevel);

  if (row >= nextLevel.height) {
    const rowsToAppend = row - nextLevel.height + 1;
    for (let index = 0; index < rowsToAppend; index += 1) {
      nextLevel.tiles.push(Array.from({ length: nextLevel.width }, createDefaultTile));
    }
  }

  if (col >= nextLevel.width) {
    const columnsToAppend = col - nextLevel.width + 1;
    nextLevel.tiles = nextLevel.tiles.map((existingRow) => [
      ...existingRow,
      ...Array.from({ length: columnsToAppend }, createDefaultTile),
    ]);
  }

  synchronizeLevelDimensions(nextLevel);

  nextLevel.tiles[row][col] = {
    sprite: { ...brush.sprite },
    walkable: brush.walkable,
    zLayer: brush.zLayer,
    effects: brush.effects.map(cloneEffect),
    tag: brush.tag.trim().length > 0 ? brush.tag.trim() : null,
  };

  return nextLevel;
}

function describeEffects(effects: LevelTileEffect[]) {
  if (effects.length === 0) {
    return "none";
  }

  return effects.map((effect) => effect.tileEffectType).join(", ");
}

function getPreferredLevel(
  availableLevelNames: string[],
  currentLevelName: string | null,
): string | null {
  const url = new URL(window.location.href);
  const requestedLevel = url.searchParams.get("level");

  if (requestedLevel && availableLevelNames.includes(requestedLevel)) {
    return requestedLevel;
  }

  if (currentLevelName && availableLevelNames.includes(currentLevelName)) {
    return currentLevelName;
  }

  if (availableLevelNames.includes("main")) {
    return "main";
  }

  return availableLevelNames[0] ?? null;
}

function syncUrl(levelName: string | null) {
  const url = new URL(window.location.href);
  if (levelName) {
    url.searchParams.set("level", levelName);
  } else {
    url.searchParams.delete("level");
  }

  window.history.replaceState({}, "", url);
}

function getGridPositionFromPointer(
  canvas: HTMLCanvasElement,
  clientX: number,
  clientY: number,
  zoom: number,
  viewOffset: ViewOffset,
): GridPosition {
  const bounds = canvas.getBoundingClientRect();
  const worldTileSize = TILESETS.pastoral.tileSize * zoom;
  const localX = clientX - bounds.left - viewOffset.x;
  const localY = clientY - bounds.top - viewOffset.y;

  return {
    col: Math.floor(localX / worldTileSize),
    row: Math.floor(localY / worldTileSize),
  };
}

function getCenteredOffset(level: LevelData, viewportSize: ViewportSize, zoom: number): ViewOffset {
  const worldTileSize = TILESETS.pastoral.tileSize * zoom;
  const levelWidth = level.width * worldTileSize;
  const levelHeight = level.height * worldTileSize;

  return {
    x: Math.round((viewportSize.width - levelWidth) * 0.5),
    y: Math.round((viewportSize.height - levelHeight) * 0.5),
  };
}

export function App() {
  const [levelNames, setLevelNames] = useState<string[]>([]);
  const [currentLevelName, setCurrentLevelName] = useState<string | null>(null);
  const [currentLevel, setCurrentLevel] = useState<LevelData | null>(null);
  const [zoom, setZoom] = useState(4);
  const [showGrid, setShowGrid] = useState(true);
  const [status, setStatus] = useState("Connecting…");
  const [tileMeta, setTileMeta] = useState(DEFAULT_TILE_META);
  const [levelErrors, setLevelErrors] = useState<string[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [cacheBust, setCacheBust] = useState(() => Date.now());
  const [brush, setBrush] = useState<Brush>(DEFAULT_BRUSH);
  const [dirty, setDirty] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [viewOffset, setViewOffset] = useState<ViewOffset>({ x: 96, y: 96 });
  const [viewportSize, setViewportSize] = useState<ViewportSize>({ width: 960, height: 640 });
  const [hoveredGrid, setHoveredGrid] = useState<GridPosition | null>(null);

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const paletteCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const viewerViewportRef = useRef<HTMLDivElement | null>(null);
  const interactionRef = useRef<InteractionState>({ mode: "idle" });
  const currentLevelRef = useRef<LevelData | null>(null);
  const currentLevelNameRef = useRef<string | null>(null);
  const brushRef = useRef<Brush>(DEFAULT_BRUSH);
  const zoomRef = useRef(zoom);
  const viewOffsetRef = useRef(viewOffset);
  const dirtyRef = useRef(dirty);
  const isSavingRef = useRef(isSaving);
  const hasUserPannedRef = useRef(false);
  const loadTilesetImage = useLevelImages(cacheBust);

  useEffect(() => {
    currentLevelRef.current = currentLevel;
  }, [currentLevel]);

  useEffect(() => {
    currentLevelNameRef.current = currentLevelName;
  }, [currentLevelName]);

  useEffect(() => {
    brushRef.current = brush;
  }, [brush]);

  useEffect(() => {
    zoomRef.current = zoom;
  }, [zoom]);

  useEffect(() => {
    viewOffsetRef.current = viewOffset;
  }, [viewOffset]);

  useEffect(() => {
    dirtyRef.current = dirty;
  }, [dirty]);

  useEffect(() => {
    isSavingRef.current = isSaving;
  }, [isSaving]);

  useEffect(() => {
    const viewport = viewerViewportRef.current;
    if (!viewport) {
      return;
    }

    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) {
        return;
      }

      setViewportSize({
        width: Math.max(1, Math.round(entry.contentRect.width)),
        height: Math.max(1, Math.round(entry.contentRect.height)),
      });
    });

    resizeObserver.observe(viewport);
    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  const levelMeta = useMemo(() => {
    if (!currentLevel) {
      return "No level loaded.";
    }

    return `${currentLevel.name} • ${currentLevel.width}x${currentLevel.height} • spawn (${currentLevel.playerSpawn.x}, ${currentLevel.playerSpawn.y})${dirty ? " • unsaved" : ""}`;
  }, [currentLevel, dirty]);

  const renderLevel = async (level: LevelData | null) => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    const canvasWidth = Math.max(1, viewportSize.width);
    const canvasHeight = Math.max(1, viewportSize.height);
    canvas.width = canvasWidth;
    canvas.height = canvasHeight;
    context.imageSmoothingEnabled = false;
    context.clearRect(0, 0, canvasWidth, canvasHeight);

    if (!level) {
      return;
    }

    const worldTileSize = TILESETS.pastoral.tileSize * zoom;
    const normalizedOffsetX = ((viewOffset.x % worldTileSize) + worldTileSize) % worldTileSize;
    const normalizedOffsetY = ((viewOffset.y % worldTileSize) + worldTileSize) % worldTileSize;

    if (showGrid) {
      context.strokeStyle = "rgba(255, 255, 255, 0.08)";
      context.lineWidth = 1;

      for (let x = normalizedOffsetX; x <= canvasWidth; x += worldTileSize) {
        context.beginPath();
        context.moveTo(Math.round(x) + 0.5, 0);
        context.lineTo(Math.round(x) + 0.5, canvasHeight);
        context.stroke();
      }

      for (let y = normalizedOffsetY; y <= canvasHeight; y += worldTileSize) {
        context.beginPath();
        context.moveTo(0, Math.round(y) + 0.5);
        context.lineTo(canvasWidth, Math.round(y) + 0.5);
        context.stroke();
      }
    }

    const usedTilesets = [
      ...new Set(level.tiles.flat().map((tile) => tile.sprite.tileset)),
    ] as TilesetName[];
    const imageEntries = await Promise.all(
      usedTilesets.map(async (tilesetName) => [
        tilesetName,
        await loadTilesetImage(tilesetName),
      ]),
    );
    const tilesetImages = new Map(imageEntries);

    const drawCalls: Array<{ row: number; col: number; tile: LevelTile }> = [];
    level.tiles.forEach((row, rowIndex) => {
      row.forEach((tile, colIndex) => {
        drawCalls.push({
          row: rowIndex,
          col: colIndex,
          tile,
        });
      });
    });

    drawCalls.sort((left, right) => {
      if (left.tile.zLayer !== right.tile.zLayer) {
        return left.tile.zLayer - right.tile.zLayer;
      }

      if (left.row !== right.row) {
        return left.row - right.row;
      }

      return left.col - right.col;
    });

    for (const drawCall of drawCalls) {
      const { tile, row, col } = drawCall;
      const tilesetConfig = TILESETS[tile.sprite.tileset];
      const tilesetImage = tilesetImages.get(tile.sprite.tileset);
      if (!tilesetImage) {
        continue;
      }

      const destinationX = viewOffset.x + col * worldTileSize;
      const destinationY = viewOffset.y + row * worldTileSize;

      if (
        destinationX + worldTileSize < 0 ||
        destinationY + worldTileSize < 0 ||
        destinationX > canvasWidth ||
        destinationY > canvasHeight
      ) {
        continue;
      }

      context.drawImage(
        tilesetImage,
        tile.sprite.col * tilesetConfig.tileSize,
        tile.sprite.row * tilesetConfig.tileSize,
        tilesetConfig.tileSize,
        tilesetConfig.tileSize,
        destinationX,
        destinationY,
        worldTileSize,
        worldTileSize,
      );
    }

    context.strokeStyle = "rgba(255, 255, 255, 0.3)";
    context.lineWidth = 2;
    context.strokeRect(
      Math.round(viewOffset.x) + 0.5,
      Math.round(viewOffset.y) + 0.5,
      level.width * worldTileSize,
      level.height * worldTileSize,
    );

    if (hoveredGrid) {
      context.strokeStyle = "rgba(96, 165, 250, 0.95)";
      context.lineWidth = 2;
      context.strokeRect(
        Math.round(viewOffset.x + hoveredGrid.col * worldTileSize) + 0.5,
        Math.round(viewOffset.y + hoveredGrid.row * worldTileSize) + 0.5,
        worldTileSize,
        worldTileSize,
      );
    }

    const spawnX = viewOffset.x + level.playerSpawn.x * worldTileSize + worldTileSize * 0.5;
    const spawnY = viewOffset.y + level.playerSpawn.y * worldTileSize + worldTileSize * 0.5;
    const markerRadius = Math.max(4, Math.round(worldTileSize * 0.18));

    context.fillStyle = "#ef4444";
    context.beginPath();
    context.arc(spawnX, spawnY, markerRadius, 0, Math.PI * 2);
    context.fill();

    context.strokeStyle = "rgba(255, 255, 255, 0.9)";
    context.lineWidth = 2;
    context.beginPath();
    context.moveTo(spawnX - markerRadius * 1.5, spawnY);
    context.lineTo(spawnX + markerRadius * 1.5, spawnY);
    context.moveTo(spawnX, spawnY - markerRadius * 1.5);
    context.lineTo(spawnX, spawnY + markerRadius * 1.5);
    context.stroke();
  };

  const renderPalette = async () => {
    const canvas = paletteCanvasRef.current;
    if (!canvas) {
      return;
    }

    const image = await loadTilesetImage("pastoral");
    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    canvas.width = image.naturalWidth * PALETTE_SCALE;
    canvas.height = image.naturalHeight * PALETTE_SCALE;
    context.imageSmoothingEnabled = false;
    context.clearRect(0, 0, canvas.width, canvas.height);
    context.drawImage(image, 0, 0, canvas.width, canvas.height);

    const selectionSize = TILESETS.pastoral.tileSize * PALETTE_SCALE;
    context.strokeStyle = "#ef4444";
    context.lineWidth = 3;
    context.strokeRect(
      brush.sprite.col * selectionSize + 1.5,
      brush.sprite.row * selectionSize + 1.5,
      selectionSize - 3,
      selectionSize - 3,
    );
  };

  const refreshLevelList = async () => {
    const payload = await fetchJson<LevelApiSummary>("/api/levels");
    setLevelNames(payload.levels);
    setLevelErrors(payload.errors.map((error) => `${error.level}: ${error.message}`));
    return payload.levels;
  };

  const loadLevel = async (levelName: string | null, shouldCenter = true) => {
    if (!levelName) {
      setCurrentLevelName(null);
      setCurrentLevel(null);
      syncUrl(null);
      setStatus("No levels found");
      setDirty(false);
      return;
    }

    setStatus(`Loading ${levelName}…`);
    const level = await fetchJson<LevelData>(`/api/levels/${encodeURIComponent(levelName)}`);
    setLoadError(null);
    setCurrentLevelName(levelName);
    setCurrentLevel(level);
    setDirty(false);
    syncUrl(levelName);

    if (shouldCenter) {
      setViewOffset(getCenteredOffset(level, viewportSize, zoomRef.current));
      hasUserPannedRef.current = false;
    }

    setStatus(`Watching ${levelName}`);
  };

  const refreshViewer = async (shouldCenter = false) => {
    const availableLevelNames = await refreshLevelList();
    const nextLevel = getPreferredLevel(availableLevelNames, currentLevelNameRef.current);
    await loadLevel(nextLevel, shouldCenter);
  };

  const paintCurrentTile = (gridPosition: GridPosition) => {
    const level = currentLevelRef.current;
    const currentBrush = brushRef.current;
    if (!level) {
      return;
    }

    const nextLevel = applyBrushToLevel(level, gridPosition, currentBrush);
    currentLevelRef.current = nextLevel;
    setCurrentLevel(nextLevel);
    setDirty(true);
  };

  const updateHoverMeta = (gridPosition: GridPosition) => {
    const level = currentLevelRef.current;
    setHoveredGrid(gridPosition);

    if (
      !level ||
      gridPosition.row < 0 ||
      gridPosition.col < 0 ||
      gridPosition.row >= level.height ||
      gridPosition.col >= level.width
    ) {
      setTileMeta(`tile (${gridPosition.col}, ${gridPosition.row}) • outside current level`);
      return;
    }

    const tile = level.tiles[gridPosition.row][gridPosition.col];
    setTileMeta(
      `tile (${gridPosition.col}, ${gridPosition.row}) • ${tile.sprite.tileset} [${tile.sprite.col}, ${tile.sprite.row}] • walkable: ${tile.walkable} • effects: ${describeEffects(tile.effects)} • tag: ${tile.tag ?? "none"}`,
    );
  };

  useEffect(() => {
    void renderLevel(currentLevel);
  }, [currentLevel, zoom, showGrid, cacheBust, viewOffset, viewportSize, hoveredGrid]);

  useEffect(() => {
    void renderPalette();
  }, [cacheBust, brush.sprite.col, brush.sprite.row]);

  useEffect(() => {
    void refreshViewer(true).catch((error: Error) => {
      setLoadError(error.message);
      setStatus("Startup failed");
    });
  }, []);

  useEffect(() => {
    const eventSource = new EventSource("/events");

    eventSource.addEventListener("ready", () => {
      setStatus((previousStatus) =>
        currentLevelNameRef.current ? `Watching ${currentLevelNameRef.current}` : previousStatus,
      );
    });

    eventSource.addEventListener("reload", (event) => {
      const payload = JSON.parse((event as MessageEvent<string>).data) as ReloadEventPayload;
      const currentName = currentLevelNameRef.current;
      const currentLevelFile = currentName ? `assets/levels/${currentName}.json` : null;
      const isCurrentLevelFile = payload.filePath === currentLevelFile;
      const isTilesetFile = payload.filePath === "assets/pastoral-tileset.png";
      const isAnyLevelFile = payload.filePath.startsWith("assets/levels/");

      if (isTilesetFile) {
        setCacheBust(Date.now());
        return;
      }

      if (dirtyRef.current && !isSavingRef.current && isAnyLevelFile) {
        void refreshLevelList();
        setStatus(`External level change detected; save or refresh when ready`);
        return;
      }

      if (isAnyLevelFile) {
        void refreshViewer(false)
          .then(() => {
            setStatus((previousStatus) => {
              if (isCurrentLevelFile && currentLevelNameRef.current) {
                return `Reloaded ${currentLevelNameRef.current}`;
              }

              return previousStatus;
            });
          })
          .catch((error: Error) => {
            setLoadError(error.message);
            setStatus("Reload failed");
          });
      }
    });

    eventSource.onerror = () => {
      setStatus("Reconnecting…");
    };

    return () => {
      eventSource.close();
    };
  }, []);

  const handleSave = async () => {
    if (!currentLevelName || !currentLevel) {
      return;
    }

    try {
      setIsSaving(true);
      setStatus(`Saving ${currentLevelName}…`);
      const response = await sendJson<SaveLevelResponse>(
        `/api/levels/${encodeURIComponent(currentLevelName)}`,
        "PUT",
        currentLevel,
      );

      currentLevelRef.current = response.level;
      setCurrentLevel(response.level);
      setDirty(false);
      setLoadError(null);
      setStatus(`Saved ${response.savedPath}`);
    } catch (error) {
      setLoadError((error as Error).message);
      setStatus("Save failed");
    } finally {
      setIsSaving(false);
    }
  };

  const handleCanvasPointerDown = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    canvas.setPointerCapture(event.pointerId);
    const gridPosition = getGridPositionFromPointer(
      canvas,
      event.clientX,
      event.clientY,
      zoomRef.current,
      viewOffsetRef.current,
    );

    if (event.button === 1) {
      event.preventDefault();
      interactionRef.current = {
        mode: "pan",
        pointerId: event.pointerId,
        lastClientX: event.clientX,
        lastClientY: event.clientY,
      };
      return;
    }

    if (event.button === 2) {
      event.preventDefault();
      const level = currentLevelRef.current;
      if (
        level &&
        gridPosition.row >= 0 &&
        gridPosition.col >= 0 &&
        gridPosition.row < level.height &&
        gridPosition.col < level.width
      ) {
        const tile = level.tiles[gridPosition.row][gridPosition.col];
        setBrush({
          sprite: { ...tile.sprite },
          walkable: tile.walkable,
          zLayer: tile.zLayer,
          effects: tile.effects.map(cloneEffect),
          tag: tile.tag ?? "",
        });
        setStatus(`Picked tile (${gridPosition.col}, ${gridPosition.row})`);
      }
      return;
    }

    if (event.button !== 0) {
      return;
    }

    event.preventDefault();
    paintCurrentTile(gridPosition);
    interactionRef.current = {
      mode: "paint",
      pointerId: event.pointerId,
      lastPaintKey: `${gridPosition.col}:${gridPosition.row}`,
    };
  };

  const handleCanvasPointerMove = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const gridPosition = getGridPositionFromPointer(
      canvas,
      event.clientX,
      event.clientY,
      zoomRef.current,
      viewOffsetRef.current,
    );

    updateHoverMeta(gridPosition);

    if (
      interactionRef.current.mode === "pan" &&
      interactionRef.current.pointerId === event.pointerId
    ) {
      event.preventDefault();
      const deltaX = event.clientX - interactionRef.current.lastClientX;
      const deltaY = event.clientY - interactionRef.current.lastClientY;

      interactionRef.current = {
        ...interactionRef.current,
        lastClientX: event.clientX,
        lastClientY: event.clientY,
      };

      hasUserPannedRef.current = true;
      setViewOffset((previousOffset) => ({
        x: previousOffset.x + deltaX,
        y: previousOffset.y + deltaY,
      }));
      return;
    }

    if (
      interactionRef.current.mode === "paint" &&
      interactionRef.current.pointerId === event.pointerId &&
      (event.buttons & 1) === 1
    ) {
      const paintKey = `${gridPosition.col}:${gridPosition.row}`;
      if (interactionRef.current.lastPaintKey !== paintKey) {
        paintCurrentTile(gridPosition);
        interactionRef.current = {
          ...interactionRef.current,
          lastPaintKey: paintKey,
        };
      }
    }
  };

  const handleCanvasPointerUp = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (
      canvasRef.current &&
      canvasRef.current.hasPointerCapture(event.pointerId)
    ) {
      canvasRef.current.releasePointerCapture(event.pointerId);
    }

    interactionRef.current = { mode: "idle" };
  };

  const handlePalettePointerDown = async (
    event: ReactPointerEvent<HTMLCanvasElement>,
  ) => {
    const canvas = paletteCanvasRef.current;
    if (!canvas) {
      return;
    }

    event.preventDefault();
    const bounds = canvas.getBoundingClientRect();
    const scaledTileSize = TILESETS.pastoral.tileSize * PALETTE_SCALE;
    const col = Math.floor((event.clientX - bounds.left) / scaledTileSize);
    const row = Math.floor((event.clientY - bounds.top) / scaledTileSize);

    if (
      col < 0 ||
      row < 0 ||
      col >= TILESETS.pastoral.columns ||
      row >= TILESETS.pastoral.rows
    ) {
      return;
    }

    setBrush((previousBrush) => ({
      ...previousBrush,
      sprite: {
        tileset: "pastoral",
        col,
        row,
      },
    }));
  };

  const addBrushEffect = () => {
    setBrush((previousBrush) => ({
      ...previousBrush,
      effects: [...previousBrush.effects, createDefaultEffect()],
    }));
  };

  const updateBrushEffect = (effectIndex: number, effect: LevelTileEffect) => {
    setBrush((previousBrush) => ({
      ...previousBrush,
      effects: previousBrush.effects.map((existingEffect, index) =>
        index === effectIndex ? effect : existingEffect,
      ),
    }));
  };

  const removeBrushEffect = (effectIndex: number) => {
    setBrush((previousBrush) => ({
      ...previousBrush,
      effects: previousBrush.effects.filter((_, index) => index !== effectIndex),
    }));
  };

  return (
    <main className="app-shell">
      <header className="toolbar">
        <div className="toolbar__group">
          <label className="field">
            <span>Level</span>
            <select
              value={currentLevelName ?? ""}
              onChange={(event) => {
                void loadLevel(event.target.value || null, true).catch((error: Error) => {
                  setLoadError(error.message);
                  setStatus("Load failed");
                });
              }}
            >
              {levelNames.length === 0 ? <option value="">No levels</option> : null}
              {levelNames.map((levelName) => (
                <option key={levelName} value={levelName}>
                  {levelName}
                </option>
              ))}
            </select>
          </label>

          <label className="field">
            <span>Zoom</span>
            <select
              value={String(zoom)}
              onChange={(event) => {
                setZoom(Number(event.target.value));
              }}
            >
              <option value="2">2x</option>
              <option value="4">4x</option>
              <option value="6">6x</option>
            </select>
          </label>

          <label className="checkbox">
            <input
              checked={showGrid}
              type="checkbox"
              onChange={(event) => {
                setShowGrid(event.target.checked);
              }}
            />
            <span>Grid</span>
          </label>
        </div>

        <div className="toolbar__group toolbar__group--status">
          <span className="status-pill">{status}</span>
          <button
            disabled={!currentLevel || isSaving}
            type="button"
            onClick={() => {
              void handleSave();
            }}
          >
            {isSaving ? "Saving…" : dirty ? "Save*" : "Save"}
          </button>
          <button
            type="button"
            onClick={() => {
              setCacheBust(Date.now());
              void refreshViewer(false).catch((error: Error) => {
                setLoadError(error.message);
                setStatus("Refresh failed");
              });
            }}
          >
            Refresh
          </button>
        </div>
      </header>

      <section className="panel">
        <div className="panel__meta">
          <div>{levelMeta}</div>
          <div>{tileMeta}</div>
          <div>
            Brush • [{brush.sprite.col}, {brush.sprite.row}] • walkable:{" "}
            {String(brush.walkable)} • z: {brush.zLayer} • tag:{" "}
            {brush.tag.trim().length > 0 ? brush.tag : "none"} • effects:{" "}
            {describeEffects(brush.effects)}
          </div>
          <div>Controls • left drag paints • right click picks • middle drag pans</div>
        </div>

        {loadError ? <div className="error-box">{loadError}</div> : null}

        {!loadError && levelErrors.length > 0 ? (
          <div className="error-box">{levelErrors.join("\n")}</div>
        ) : null}
      </section>

      <section className="workspace">
        <aside className="sidebar">
          <section className="editor-panel">
            <h2>Palette</h2>
            <canvas
              ref={paletteCanvasRef}
              className="palette-canvas"
              onContextMenu={(event) => {
                event.preventDefault();
              }}
              onPointerDown={(event) => {
                void handlePalettePointerDown(event);
              }}
            />
          </section>

          <section className="editor-panel">
            <h2>Brush</h2>

            <label className="field">
              <span>Walkable</span>
              <input
                checked={brush.walkable}
                type="checkbox"
                onChange={(event) => {
                  setBrush((previousBrush) => ({
                    ...previousBrush,
                    walkable: event.target.checked,
                  }));
                }}
              />
            </label>

            <label className="field">
              <span>Z Layer</span>
              <input
                step="0.1"
                type="number"
                value={brush.zLayer}
                onChange={(event) => {
                  const nextValue = Number(event.target.value);
                  setBrush((previousBrush) => ({
                    ...previousBrush,
                    zLayer: Number.isFinite(nextValue) ? nextValue : 0,
                  }));
                }}
              />
            </label>

            <label className="field">
              <span>Tag</span>
              <input
                placeholder="optional"
                type="text"
                value={brush.tag}
                onChange={(event) => {
                  setBrush((previousBrush) => ({
                    ...previousBrush,
                    tag: event.target.value,
                  }));
                }}
              />
            </label>

            <div className="effects-editor">
              <div className="effects-editor__header">
                <h3>Effects</h3>
                <button type="button" onClick={addBrushEffect}>
                  Add
                </button>
              </div>

              {brush.effects.length === 0 ? (
                <div className="empty-note">No effects</div>
              ) : (
                brush.effects.map((effect, effectIndex) => (
                  <div className="effect-row" key={effectIndex}>
                    <label className="field">
                      <span>Type</span>
                      <select
                        value={effect.tileEffectType}
                        onChange={(event) => {
                          updateBrushEffect(effectIndex, {
                            ...effect,
                            tileEffectType: event.target.value as TileEffectType,
                          });
                        }}
                      >
                        {TILE_EFFECT_TYPES.map((effectType) => (
                          <option key={effectType} value={effectType}>
                            {effectType}
                          </option>
                        ))}
                      </select>
                    </label>

                    <label className="field">
                      <span>Modifier</span>
                      <input
                        step="0.1"
                        type="number"
                        value={effect.modifier}
                        onChange={(event) => {
                          const nextModifier = Number(event.target.value);
                          updateBrushEffect(effectIndex, {
                            ...effect,
                            modifier: Number.isFinite(nextModifier) ? nextModifier : 0,
                          });
                        }}
                      />
                    </label>

                    <label className="field">
                      <span>Extra Data</span>
                      <input
                        placeholder="optional"
                        type="text"
                        value={effect.extraData ?? ""}
                        onChange={(event) => {
                          updateBrushEffect(effectIndex, {
                            ...effect,
                            extraData:
                              event.target.value.trim().length > 0
                                ? event.target.value
                                : null,
                          });
                        }}
                      />
                    </label>

                    <button
                      className="effect-row__remove"
                      type="button"
                      onClick={() => {
                        removeBrushEffect(effectIndex);
                      }}
                    >
                      Remove
                    </button>
                  </div>
                ))
              )}
            </div>
          </section>
        </aside>

        <section className="viewer-wrap viewer-wrap--editor" ref={viewerViewportRef}>
          <canvas
            id="level-canvas"
            ref={canvasRef}
            onContextMenu={(event) => {
              event.preventDefault();
            }}
            onPointerDown={handleCanvasPointerDown}
            onPointerLeave={() => {
              setTileMeta(DEFAULT_TILE_META);
              setHoveredGrid(null);
            }}
            onPointerMove={handleCanvasPointerMove}
            onPointerUp={handleCanvasPointerUp}
            onPointerCancel={handleCanvasPointerUp}
          />
        </section>
      </section>
    </main>
  );
}
