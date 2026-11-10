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
- View to the RustGIS layout, on the globe's toolbar, puts the view in a page layout as a
  picture kept in the project, as the PlanetX plugin's View to QGIS layout: in the only layout,
  else the open one, else a new A4 layout.
- RustGIS's AI Assistant drives the globe: the plugin's commands `fly` (the globe to the map's
  view tilted 60°, or to a point or an extent with a heading and a tilt) and `layout` (the view
  to a page layout), and its skill `rustplanetx:globe` telling the assistant when to use them.
  The commands are on the ribbon too, as 3D View and View to Layout.
- What the PlanetX plugin puts into the QGIS project goes to the RustGIS project, into the group
  RustPlanetX - globe layers: view objects and pictures of catalogue layers, view contours,
  placemark layers, pictures on the surface, satellite images, trend maps and Rumsey maps.
  Analysis rasters come with the colours of the globe.
- Terrain, imagery, site analysis, tours, KML and `.planetx` scenes are PlanetX Lite's.

## Use

1. Install `rustplanetx-<version>.rgplugin` in RustGIS 0.5.0 or newer:
   **Plugins › Plugin Manager › Settings › Install From File…**.
2. Click **PlanetX Globe** on the **Plugins** tab.

A RustGIS with plugin sessions ([opengeos/RustGIS#1553](https://github.com/opengeos/RustGIS/pull/1553))
links the globe at once. A RustGIS without them needs the relay: start the **MCP Relay**
command (Ctrl+K), or tick **Let your Agents Control RustGIS** in the AI Assistant's settings.
The globe then connects to the relay on `127.0.0.1:8767` (`RUSTGIS_MCP_PORT`) with the token
file RustGIS writes for its own `rustgis-cli mcp --attach`. Nothing leaves the computer.

## How it works

The plugin is a [process plugin](https://github.com/opengeos/RustGIS/blob/main/docs/plugins.md#process-plugins):
its program is PlanetX Lite itself. RustGIS runs `planetx-lite.exe rustgis-plugin`.

- With `"session": true` RustGIS keeps the program running. The globe opens in that process
  and talks to RustGIS over its standard input and output: no port, no token. It reads the
  project with `info.project` and the features with `host.features`, and rereads them when
  RustGIS sends an event. The globe closes with RustGIS.
- A RustGIS without sessions runs the program once per command. It then starts the globe as
  `planetx-lite.exe --rustgis` and returns, and the globe uses the MCP relay.

The plugin asks for `edit` (selection, visibility, layers to the project), `files` (layer
styles and raster tiles come as files, analysis rasters go as GeoTIFF) and `network` (the
globe follows tile layers with `map.setTiles`). The link is in PlanetX Lite,
`crates/ui/src/rustgislink.rs` and `crates/data/src/rustgis.rs`. A process plugin is not
sandboxed, and the install prompt says so. The package is for Windows.

## Where changes go

PlanetX comes as three products: the PlanetX plugin for QGIS, PlanetX Lite and RustPlanetX. A
feature lives in one of them, by its role. Analysis and project layers are the QGIS plugin's,
viewing, scenes and tours are Lite's, the link with RustGIS is RustPlanetX's. The `.planetx`
scene and placemark format is shared by all three, in
[planetx-scene](https://github.com/Valery35/planetx-scene). Details are in PlanetX Lite's
`doc/PLAN.md`, section «Как снять перегруз».

In PlanetX Lite, the code of RustPlanetX:

- `crates/ui/src/app/rustgis.rs` - the RustGIS section of the globe window and the link steps;
- `crates/ui/src/rustgislink.rs` and `crates/data/src/rustgis.rs` - the session and relay
  clients and the reading of RustGIS state;
- `apps/planetx-rustgis` - the program, built without terrio and add-ins.

A new layer of the globe's catalog is one line in `crates/ui/src/registry.rs`, under the
build feature of its theme (`live`, `earth`, `imagery`, `analysis`, `sky`, `ai`).
`apps/planetx-rustgis/Cargo.toml` names the themes RustPlanetX is built with.

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

## Release

```sh
powershell -File scripts/release.ps1 -Version 0.5.7 -Notes "what is new"
```

The script takes the last PlanetX Lite commit on `main` with a green CI, or `-LiteRef`, writes
it to `lite.ref` and the version to `plugin.json`, commits, tags `v<version>` and pushes. It
waits for the Release workflow, checks the package against its registry entry, copies it to
Dropbox with the version note and puts the registry entry to the branch of the open
`opengeos/rustgis-plugins` pull request, or to a new branch. `-Install` installs the release
in RustGIS, `-SkipCatalog` leaves the registry alone, `-DryRun` only checks and prints the plan.

## License

Apache 2.0, © 2026 Inform++ LLC.
