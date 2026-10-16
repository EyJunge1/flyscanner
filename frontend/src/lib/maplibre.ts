import { setWorkerUrl } from 'maplibre-gl';
import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?url';

/** Vite does not resolve the default MapLibre worker URL — set it once. */
setWorkerUrl(workerUrl);
