# Plattformen

Flyscanner (Rust-Binary + eingebettetes Frontend) läuft nativ und per Docker.

| Plattform | Nativ | Docker Demo | Docker Full (SDR) |
|-----------|-------|-------------|-------------------|
| Linux | ja | ja | ja (USB) |
| macOS | ja | ja | nein (kein zuverlässiges USB) |
| Windows 10/11 | ja (MSVC) | ja (Linux-Container) | nein |
| Windows Server | ja (MSVC) | ja, mit Linux-Containern | nein |

## Voraussetzungen (nativ)

- **Rust** stable (`rustup`)
- **Node.js** 22+ (Frontend-Build)
- **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) mit „Desktop development with C++“ (MSVC + Windows SDK). Turso zieht `mimalloc` und braucht einen C-Compiler.

### Schnellstart (alle OS)

```bash
# Frontend bauen + Demo (npm-Scripts am Repo-Root — Windows/macOS/Linux)
npm run demo
# UI: http://127.0.0.1:8080
```

Oder manuell:

```bash
cd frontend && npm ci && npm run build && cd ..
cargo run -- --demo --db data/flyscanner.db
```

Unix zusätzlich: `make demo`.

## Docker

**Demo** (überall, wo Docker Linux-Container kann):

```bash
docker compose -f deploy/docker-compose.yml --profile demo up --build
# oder: npm run docker:demo
```

**Beast von außen** (kein lokales SDR, z. B. Win/Mac → Host-Beast oder Remote):

```bash
# Host-Beast auf Port 30005 (Docker Desktop: host.docker.internal)
export FLYSCANNER_BEAST_HOST=host.docker.internal
docker compose -f deploy/docker-compose.yml --profile beast up --build
```

**Full + SDR** nur auf Linux mit USB:

```bash
docker compose -f deploy/docker-compose.yml -f deploy/docker-compose.sdr.yml \
  --profile full up --build
# oder: npm run docker:full
```

## Windows Server

1. Nativ: MSVC-Toolchain wie oben, dann `npm run demo` / `cargo run --release`.
2. Docker: **Linux-Container**-Modus (Moby/WSL2 bzw. passende Docker-Engine). Windows-Container-Images gibt es für Flyscanner nicht — das Runtime-Image ist Debian.

## CI

`.github/workflows/ci.yml` baut und testet auf **Ubuntu, macOS und Windows** und baut das Docker-Image (Ubuntu).

## Release-Binaries

Tags `v*` trigger `.github/workflows/release.yml` und erzeugen Archive für Linux x86_64, macOS (arm64 + x86_64) und Windows x86_64.
