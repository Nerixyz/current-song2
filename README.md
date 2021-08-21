# CurrentSong 2

This project is a rewrite of the [**CurrentSong Overlay**](https://github.com/Nerixyz/current-song-overlay).
The core is written in Rust now.
That doesn't change much,
but now the project supports Windows' [`GlobalSystemMediaTransportControls`](https://docs.microsoft.com/uwp/api/windows.media.control).

The goal of the project is to create a **simple yet powerful** overlay that displays the currently playing song.
There are a few unique features separating this project:

* **Near zero latency** ⏱ All modules are created with this in mind.
* **Displaying Progress** 💯 Progress is displayed where available.
* **Display Album Art** 🖼
* **Customizable** 🔧 The overlay is customizable through CSS. Modules and the server can be configured in a `config.toml` file.

# Setup

TODO

# Planned Features

* **Browser Extension** - To get progress from browsers.
* **Windows Service** 💻 To simplify usage
