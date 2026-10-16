# Flyscanner

![Flyscanner Banner](.github/pics/banner.png)

> **Beta**: breaking changes possible

ADS-B all-in-one: Beast feed → live map (Rust + Svelte 5). Demodulation stays with readsb.

## Install

Requirements: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) 22+  
(Windows also needs [VS Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++)

```bash
npm run demo
# → http://127.0.0.1:8080
```

Docker (no SDR):

```bash
npm run docker:demo
```

Node is only needed to build the UI. The finished binary runs without Node.

## License

[MIT](LICENSE)
