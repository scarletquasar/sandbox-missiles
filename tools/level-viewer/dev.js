const net = require("net");
const path = require("path");
const { spawn } = require("child_process");

const viewerRoot = __dirname;
const host = "127.0.0.1";
const viteBin = path.join(viewerRoot, "node_modules", "vite", "bin", "vite.js");

function getFreePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();

    server.on("error", reject);
    server.listen(0, host, () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("Unable to resolve a free port"));
        return;
      }

      const { port } = address;
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }

        resolve(port);
      });
    });
  });
}

function pipeWithPrefix(stream, prefix, output) {
  let buffer = "";

  stream.on("data", (chunk) => {
    buffer += chunk.toString();
    const lines = buffer.split(/\r?\n/);
    buffer = lines.pop() ?? "";

    for (const line of lines) {
      output.write(`${prefix} ${line}\n`);
    }
  });

  stream.on("end", () => {
    if (buffer.length > 0) {
      output.write(`${prefix} ${buffer}\n`);
    }
  });
}

async function main() {
  const apiPort = await getFreePort();
  let webPort = await getFreePort();
  while (webPort === apiPort) {
    webPort = await getFreePort();
  }

  console.log(`Starting level viewer with API on http://${host}:${apiPort}`);
  console.log(`Starting level viewer with web on http://${host}:${webPort}`);

  const children = [];
  let shuttingDown = false;

  const stopChildren = () => {
    if (shuttingDown) {
      return;
    }

    shuttingDown = true;
    for (const child of children) {
      if (!child.killed) {
        child.kill("SIGTERM");
      }
    }
  };

  const apiProcess = spawn(process.execPath, [path.join(viewerRoot, "level-viewer.js")], {
    cwd: viewerRoot,
    env: {
      ...process.env,
      HOST: host,
      PORT: String(apiPort),
      STRICT_PORT: "true",
    },
    stdio: ["ignore", "pipe", "pipe"],
  });

  const webProcess = spawn(
    process.execPath,
    [
      viteBin,
      "--host",
      host,
      "--port",
      String(webPort),
      "--strictPort",
    ],
    {
      cwd: viewerRoot,
      env: {
        ...process.env,
        LEVEL_VIEWER_API_PORT: String(apiPort),
      },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  children.push(apiProcess, webProcess);

  pipeWithPrefix(apiProcess.stdout, "[api]", process.stdout);
  pipeWithPrefix(apiProcess.stderr, "[api]", process.stderr);
  pipeWithPrefix(webProcess.stdout, "[web]", process.stdout);
  pipeWithPrefix(webProcess.stderr, "[web]", process.stderr);

  const exitHandler = (signal) => {
    stopChildren();
    process.exitCode = signal === "SIGINT" ? 130 : 143;
  };

  process.on("SIGINT", exitHandler);
  process.on("SIGTERM", exitHandler);

  for (const [name, child] of [
    ["api", apiProcess],
    ["web", webProcess],
  ]) {
    child.on("exit", (code, signal) => {
      if (!shuttingDown) {
        stopChildren();

        if (signal) {
          console.error(`[${name}] exited with signal ${signal}`);
          process.exitCode = 1;
        } else if (code && code !== 0) {
          console.error(`[${name}] exited with code ${code}`);
          process.exitCode = code;
        } else {
          process.exitCode = 0;
        }
      }
    });
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
