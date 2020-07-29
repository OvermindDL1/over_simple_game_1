# Simple game for `shipyard` testing

To run it in debug just clone it and run `cargo run`, or `cargo run --release` for release mode.

The purpose of this project is mainly to learn the `shipyard` library as a replacement for my own.  It may grow to more in time but for now that is its focus.

It is split into a library (might be pulled standalone later) that contains all the game data with nothing known about the renderer or interfaces, just an event pump essentially, and a front-end that is currently made in GGEZ, might be replaced with something else later so 3D can be used, but it just takes the map data and renders it as appropriate, passing events to the engine, etc...

Currently the engine supports any amount of maps indexed by string name, each map can be up to 256x256 tiles in side, optionally can wrap around X (not Y), and the tiles are hex grides, the maps exists as a hex based rhombus.

Currently no textures or data files are included so trying to run it will fail with an error message about what it was unable to do, but you could potentially create your own until I include basic data.
