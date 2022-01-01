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

## Windows Autostart

To remove the application from autostart, run `current-song2.exe --remove-autostart` from a terminal.

Alternatively you can **disable** the autostart entry in the Task Manager (startup tab).

# Planned Features

See more in the [projects tab](https://github.com/Nerixyz/current-song2/projects/1).

* **Browser Extension** - To get progress from browsers.
* **Windows Service** 💻 To simplify usage
* **Easy setup**
* **Single executable**
* **Output to File**
