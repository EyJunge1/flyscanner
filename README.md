# Flyscanner

![Flyscanner Banner](.github/pics/banner.png)

<!-- Social preview: .github/pics/social-preview.png (2560×1280 / 2×) — GitHub → Settings → General → Social preview -->


> **Warning — Beta / Fun-Projekt**  
> Noch in der **Beta**, wird derzeit **nicht aktiv weiterentwickelt** — aktuell eher ein **Spaß-/Hobby-Projekt**. Erwarte Breaking Changes, fehlende Features und keine Support-Garantie.

Modernes ADS-B All-in-One: **Rust** (Beast → rs1090 → State → axum) + **Svelte 5** (MapLibre). History lokal mit **Turso/libSQL**. Demodulation bleibt bei **readsb**.

Läuft auf **Linux, macOS, Windows / Windows Server** (nativ) und **Docker**. Details: [docs/PLATFORMS.md](docs/PLATFORMS.md). UI-Farben: [docs/COLORS.md](docs/COLORS.md) (opencode-mem Dark-Shell, Accent Sky-Blau statt Lila).

![](https://img.shields.io/badge/-0a0a0a?style=flat-square&color=0a0a0a)
![](https://img.shields.io/badge/-141414?style=flat-square&color=141414)
![](https://img.shields.io/badge/-2a2a2a?style=flat-square&color=2a2a2a)
![](https://img.shields.io/badge/-d4d4d4?style=flat-square&color=d4d4d4)
![](https://img.shields.io/badge/-8a8a8a?style=flat-square&color=8a8a8a)
![](https://img.shields.io/badge/-38bdf8?style=flat-square&color=38bdf8)
![](https://img.shields.io/badge/-34d399?style=flat-square&color=34d399)
![](https://img.shields.io/badge/-f44336?style=flat-square&color=f44336)
![](https://img.shields.io/badge/-f0b429?style=flat-square&color=f0b429)

## Quick start

Cross-platform (Windows / macOS / Linux):

```bash
npm run demo
# UI http://127.0.0.1:8080
```

**Build:** Node nur zum Kompilieren der Svelte-UI (`frontend/` → `frontend/build`). Die fertige Binary (und Docker-Images) brauchen **kein Node** — die UI steckt per `rust-embed` im Rust-Binary.

Unix-Make-Äquivalent: `make demo`.

```bash
# Frontend-Dev (Vite-Proxy → :8080)
npm run frontend:dev

# Frontend-Sim (nur Vite, synthetische Flugzeuge — kein Backend)
npm run frontend:sim
```

### Windows (nativ)

1. [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) mit C++-Workload
2. [rustup](https://rustup.rs/) + Node.js 22+
3. PowerShell: `npm run demo` oder `.\scripts\demo.ps1`

## Docker

```bash
# Demo (ohne SDR) — Linux / macOS / Windows
npm run docker:demo

# Beast vom Host/Remote (kein USB), z. B. Docker Desktop:
#   FLYSCANNER_BEAST_HOST=host.docker.internal npm run docker:beast

# Full-Stack mit SDR — nur Linux + USB:
npm run docker:full
```

Oder:

```bash
cp .env.example .env
docker compose -f deploy/docker-compose.yml --profile demo up -d --build
```

## Aircraft-DB

```bash
mkdir -p data
curl -L -o data/aircraft.csv.gz \
  https://github.com/wiedehopf/tar1090-db/raw/csv/aircraft.csv.gz
cargo run -- --aircraft-db data/aircraft.csv.gz --demo
```

## Feeding

```bash
# Outbound Beast an Aggregatoren (in-process):
export FLYSCANNER_CONFIG="adsb,feed.adsb.fi,30004,beast_reduce_plus_out;adsb,in.adsb.lol,30004,beast_reduce_plus_out"
cargo run -- --beast-host 127.0.0.1 --beast-port 30005

# Oder Sidecars PiAware / FR24 / ADSBx (Linux Full-Stack):
docker compose -f deploy/docker-compose.yml -f deploy/docker-compose.sdr.yml \
  -f deploy/docker-compose.feeders.yml \
  --profile full --profile feeders up -d --build
```

Beast-out lauscht default auf `0.0.0.0:30004` (mit `--no-beast-out` abschaltbar).

## UI

**Vollkarte** + floating Toolbar.

- **Super Filter**: Alt/GS/Dist/Source/Type/Reg/Squawk/Search, Isolate, Select-all
- **InfoBlock** (Auswahl): Identity / Spatial / Signal + **Altitude-Lifeline**
- **Bottom-Panel**: Search · Filters · Columns · Stats + sortierbare Tabelle
- Layer: Trails (Lifelines) / Labels / Ext / Rings / Heat / pTracks / Basemap / Dim
- Follow / Home / Isolate, Hover-Info, KML, Permalink-URL
- Shortcuts: `T` `L` `O` `R` `H` `P` `F` `G` `I` `M` `B`, `Space`/`←`/`→`, `?`, `Esc`

## Lint

```bash
npm run check
# oder: make fmt && make check
```

## API

| Endpoint | Beschreibung |
|----------|--------------|
| `GET /api/aircraft` | Live-Liste (`min_alt`, `box`, `circle`, `closest`, `limit`) |
| `GET /api/aircraft/{icao}/track` | Track (`since`/`until` oder `day=YYYY-MM-DD`) |
| `GET /api/history/ptracks` | Persistente Tracks |
| `GET /api/history/heatmap` | Heatmap |
| `GET /api/history/replay` | Replay-Snapshots |
| `GET /api/history/trace/{icao}?day=` | Tages-Trace (UTC) |
| `GET /api/stats/series` | Chart-Serien |
| `GET /metrics` | Prometheus |
| `WS /ws` | Live: `snapshot` / `delta` / `stats` |
