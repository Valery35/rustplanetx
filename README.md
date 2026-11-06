# RustPlanetX

RustPlanetX links [RustGIS](https://github.com/opengeos/RustGIS) with the
[PlanetX Lite](https://github.com/Valery35/planetx-lite) 3D globe, as the PlanetX plugin links
QGIS with its globe.

- The layers of the RustGIS project are listed in the globe's RustGIS project section, as the
  PlanetX plugin lists the QGIS project's layers. They start switched off: a check shows a
  layer on the globe in the colours of its RustGIS renderer (single symbol, graduated and
  unique values). The list follows the project: a layer added, removed or restyled in RustGIS
  changes on the globe within a second.
- The globe follows the RustGIS map, and the map follows the globe once it stops moving.
- A feature clicked on the globe with Identify is selected in RustGIS.
- A layer of the globe goes to the RustGIS project from its menu, To the RustGIS project.
- Terrain, imagery, site analysis, tours, KML and `.planetx` scenes are PlanetX Lite's.

## Use

1. Install `rustplanetx-<version>.rgplugin` in RustGIS 0.5.0 or newer:
   **Plugins › Plugin Manager › Settings › Install From File…**.
2. In RustGIS start the **MCP Relay** command (Ctrl+K), or tick **Let your Agents Control
   RustGIS** in the AI Assistant's settings: RustGIS then starts the relay with the app.
3. Click **PlanetX Globe** on the **Plugins** tab.

The globe connects to the relay on `127.0.0.1:8767` (`RUSTGIS_MCP_PORT`) with the token file
RustGIS writes for its own `rustgis-cli mcp --attach`. Nothing leaves the computer.

## How it works

The plugin is a [process plugin](https://github.com/opengeos/RustGIS/blob/main/docs/plugins.md#process-plugins):
its program is PlanetX Lite itself. RustGIS runs `planetx-lite.exe rustgis-plugin`, which
starts the globe as `planetx-lite.exe --rustgis` and returns. The link is in PlanetX Lite,
`crates/ui/src/rustgislink.rs` and `crates/data/src/rustgis.rs`. A process plugin is not
sandboxed, and the install prompt says so. The package is for Windows.

## Packaging

```sh
powershell -File scripts/package.ps1 -Lite path\to\planetx-lite.exe
```

It writes `dist/rustplanetx-<version>.rgplugin`, the folder `dist/rustplanetx/` for
**Add Plugin Folder…**, and the registry entry `dist/rustplanetx.json`.

A tag `v<version>` starts the Release workflow. It builds `rustplanetx.exe` from the
PlanetX Lite commit named in `lite.ref` (`cargo build --release -p planetx-rustgis`),
packages it, runs the registry checks of `opengeos/rustgis-plugins` and attaches the package
and its registry entry to the GitHub release.

## License

Apache 2.0, © 2026 Inform++ LLC.
