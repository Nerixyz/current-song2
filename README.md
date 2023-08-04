# Current Song 2

![example screenshot with default theme](https://github.com/Nerixyz/current-song2/assets/19953266/9b2ac5cd-4135-4eea-8383-bc738c865da9)

The documentation is located at [currentsong.nerixyz.de](https://currentsong.nerixyz.de).

This project is a rewrite of the [**CurrentSong Overlay**](https://github.com/Nerixyz/current-song-overlay). The core is
written in Rust now. That doesn't change much, but now the project supports
Windows' [`GlobalSystemMediaTransportControls`](https://docs.microsoft.com/uwp/api/windows.media.control).

The goal of the project is to create a **simple yet powerful** overlay that displays the currently playing song. There
are a few unique features separating this project:

- **Near zero latency** ⏱ Current Song 2 doesn't poll applications or APIs for updates.
- **Displaying Progress** 💯 Progress is displayed where available.
- **Display Album Art** 🖼
- **Customizable** 🔧 The overlay is customizable through CSS (`theme.css`) and JavaScript (`user.js`), see [Customization]. Modules and the server can be configured in
  a `config.toml` file, see [Configuration].

- [Setup](https://currentsong.nerixyz.de/#getting-started)
- [Configuration]
- [Customization]
- [Building](https://currentsong.nerixyz.de/Building)

## Architecture

```mermaid
graph TD;
    gsmtc[Windows GSMTC]
    ext[Browser Extension]
    cso2[CurrentSong2]
    ext-->cso2
    Spotify-->gsmtc
    Browser-- no playback progress -->gsmtc
    Browser-- playback progress -->ext
    gsmtc-->cso2
    cso2-->Overlay
```

## Planned Features

See more in the [projects tab](https://github.com/Nerixyz/current-song2/projects/1).

[Customization]: https://currentsong.nerixyz.de/Customization
[Configuration]: https://currentsong.nerixyz.de/Configuration
