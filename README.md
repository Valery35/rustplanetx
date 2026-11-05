# RustPlanetX

RustPlanetX is a [RustGIS](https://github.com/opengeos/RustGIS) plugin that links RustGIS to
[PlanetX Lite](https://valery35.github.io/planetx-lite/), a 3D globe in the browser with site
analysis, multimedia tours, KML and `.planetx` scenes.

- **Open in PlanetX Lite** builds a link to PlanetX Lite at the map's view:
  `https://valery35.github.io/planetx-lite/?pose=latitude,longitude,distance,0,0`. The camera
  distance gives the same ground metres per screen pixel as the map's scale. A RustGIS plugin
  cannot open a browser, so the link appears in the panel and in the status bar. Copy it into
  the browser.
- **Catalogue** in the PlanetX panel adds PlanetX sources as tile layers:
  - 19 ready basemaps (OpenStreetMap styles, Esri, NASA, USGS);
  - 24 NASA GIBS layers on the day chosen with the slider (clouds, smoke, snow, gases, night
    lights and more);
  - Esri World Imagery Wayback releases since 2014;
  - imagery of the Moon, Mars, Mercury, Venus, moons of Jupiter and Saturn, Ceres and Vesta.

The panel, the day slider and the Wayback list need RustGIS 0.5.0. From scripts and
`rustgis-cli` every source can be added by its id:

```sh
rustgis-cli run --plugin dist/rustplanetx/ --sample \
  --cmd 'plugin:rustplanetx/open-in-lite' \
  --cmd 'plugin:rustplanetx/add-source={"source": "gibs:snow", "panel": {"day": 362}}' \
  --cmd 'plugin:rustplanetx/add-source={"source": "body:mars"}'
```

The day slider runs from 0, a year back, to 365, the last ready day: 362 is three days
before it. Source ids: `ready:<key>`, `gibs:<key>`, `wayback:<release>:<day>`, `body:<key>`, listed in
`src/catalog.rs`.

## Permissions

`edit` and `network`: the plugin adds tile layers (`map.addTiles`) and reads the GIBS date
series and the Wayback release list (`host.fetch`).

## Building

```sh
rustup target add wasm32-unknown-unknown
cargo test
powershell -File scripts/package.ps1
```

The script builds the module and writes `dist/rustplanetx-<version>.rgplugin`, the folder
`dist/rustplanetx/` for **Plugin Manager › Settings › Add Plugin Folder…** and the registry entry
`dist/rustplanetx.json`.

## License

Apache 2.0, © 2026 Inform++ LLC. The sources' addresses come from PlanetX Lite. The tiles
belong to their providers, and each layer carries its attribution.
