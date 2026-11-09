---
name: globe
description: Show a place of the map on the PlanetX 3D globe of the RustPlanetX plugin, with terrain and at an angle, and put the globe's view in a page layout as a picture. Use when the user asks for a 3D view, a view at an angle or from the side, the relief or mountains in 3D, a globe, or a 3D picture of the map in a layout.
---

# PlanetX globe

The RustPlanetX plugin links the map with the PlanetX 3D globe: terrain, imagery and the
project's layers in their RustGIS colours. Its commands:

- `plugin:rustplanetx/globe` `{}` opens the globe. Run it first: the other commands need the
  globe open and fail with a message saying so.
- `plugin:rustplanetx/fly` flies the globe and answers its new `lat`, `lon`, `distance`,
  `heading` and `tilt`.
  - `{}`: to the map's current view, tilted 60 degrees. The simplest way to show what the map
    shows in 3D.
  - `{"lat": 43.35, "lon": 42.44, "distance": 15000}`: to a point, `distance` in metres from the
    eye. Without `distance` the globe keeps its current one.
  - `{"extent": [west, south, east, north]}`: to an extent in degrees, such as a layer's.
  - `"heading"` (degrees clockwise from north, the direction the camera looks) and `"tilt"`
    (0 looks straight down, 60 to 75 shows relief well, 90 looks at the horizon) go with any of
    them. Without them the globe keeps its own, except the map's view, which is tilted 60.
- `plugin:rustplanetx/layout` `{}` puts the globe's view in a page layout as a picture kept in
  the project: in the only layout, else the open one, else a new A4 layout named PlanetX. It
  waits for the globe to stop and load the place, so run it after `fly`, not instead of it.
  It answers the layout's name, or `null` for the open layout.

## Recipes

- "Show this area in 3D": `globe`, then `fly` `{}`, or `fly` with the area's extent and a
  `"tilt"` of 60 to 70.
- "Look at the mountain from the south": `fly` with its point, `"heading": 0` and `"tilt": 70`.
  The camera looks north, so it stands to the south.
- "Add a 3D view to the layout": `fly` to the place, then `layout`.

Tell the user what the globe shows. Layers of the project are on the globe when the user ticks
them in its RustGIS project section, or when its view properties follow the map's layers.
