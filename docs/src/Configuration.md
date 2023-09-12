# Configuration

<!-- prettier-ignore -->
!!! note
    The config is loaded when the server is started. In order to apply the configuration, you need to **restart** the application. On Windows you should only need to double-click the `current-song2.exe` again, and it will ask you to stop the old process.

The configuration uses the [toml](https://toml.io) format. The default configuration looks like this:

```toml
no_autostart = true

# GSMTC is Windows only
[modules.gsmtc]
enabled = true

[modules.gsmtc.filter]
mode = "Exclude"
items = ["chrome.exe", "msedge.exe", "firefox.exe"]

# DBus is Unix only
[modules.dbus]
enabled = true
destinations = ["org.mpris.MediaPlayer2.spotify"]

[modules.file]
enabled = false

[server]
port = 48457
custom_theme_path = "theme.css"
```

## Location

The server executable always searches for the configuration file next to itself:

```text
╰─ MyFolder
   │
   ├─ current-song2.exe
   ╰─ config.toml
```

## `no_autostart` :fontawesome-brands-windows:

This flag controls if the application will try to add itself to autostart.

-   If it's `true`, then it won't add itself to autostart. _This doesn't mean it will be removed! See [Autostart](index.md#Autostart)._
-   If it's `false` (default), then it **will** check the autostart and possibly add itself there. You can still disable
    the entry on the _Task Manager_'s _Autostart_ tab since this is independent of the actual registry entry.

## GSMTC (Global System Media Transport Controls) :fontawesome-brands-windows:

GSMTC uses Windows' own media tracking to provide metadata. However, not every application emits metadata to this system
or only limited metadata (specifically browsers; that's why they're excluded by default).

### Filter

You can control which applications will be included in the search for metadata through `modules.gsmtc.filter`. There are
three modes: `Disabled`,`Include`, and `Exclude`:

-   `Disabled` will disable all filters, and let everything pass the filters:

    ```toml
    [modules.gsmtc.filter]
    mode = "Disabled"
    ```

-   `Include` will only include applications listed in `items`. For example, only
    include Spotify:

    ```toml
    [modules.gsmtc.filter]
    mode = "Include"
    items = ["Spotify.exe"] # (1)!
    ```

    1. Notice the capital 'S', the filter is case-sensitive.

-   `Exclude` will include everything, except applications listed in `items`. For
    example, don't include firefox:

    ```toml
    [modules.gsmtc.filter]
    mode = "Exclude"
    items = ["firefox.exe"]
    ```

<!-- prettier-ignore -->
!!! note
    Both lists are **case-sensitive**. <br/>
    You can see the application name in the _Task Manager_ by right-clicking and selecting _Properties_.

### `is_enabled`

Controls whether the module should be enabled or not.

## D-Bus :fontawesome-brands-linux:

The D-Bus module and collect metadata from any player implementing the [Media Player Remote Interfacing Specification (MPRIS)](https://specifications.freedesktop.org/mpris-spec/latest/). Most players support MPRIS (see [_Supported Clients_](https://wiki.archlinux.org/title/MPRIS#Supported_clients)). To collect metadata from a specific client, add its destination to [`destinations`](#destinations).

The following [metadata](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata) is collected (when available):

-   [`mpris:length`](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/#mpris:length)
-   [`xesam:title`](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/#xesam:title)
-   [`xesam:album`](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/#xesam:album)
-   [`xesam:trackNumber`](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/#xesam:tracknumber)
-   [`xesam:artist`](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/#xesam:artist) (joined by `, `)
-   [`mpris:artUrl`](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/#mpris:arturl)

### `destinations`

A list of destinations to listen to. Often this is the player name in lower-case prefixed with `org.mpris.MediaPlayer2.`. Each destination becomes a source formatted as `dbus::{destination}`.

### `is_enabled`

Controls whether the module should be enabled or not.

## Server

### `custom_theme_path`

Controls the path from which a CSS theme will be loaded, defaults to `theme.css`. This is intended, so that you can keep
multiple themes in the folder and switch between them.

### `custom_script_path`

Controls the path from which a user script will be loaded, defaults to `user.js`. This is intended, so that you can keep
multiple scripts in the folder and switch between them.

### `port`

Controls the local port on which the server is listening, defaults to `48457`.

<!-- prettier-ignore -->
!!! note
    If you change the port, make sure to change it in the extension as well.

## File Output

Current Song 2 can output the playing song to a file (disabled by default).
To enable file-output, set `modules.file.enabled` to `true`:

```toml
[modules.file]
enabled = true
path = "current_song.txt"     # default
format = "{artist} - {title}" # default
```

This will write to the file specified by `modules.file.path` (defaults to `current_song.txt`)
with the format specified by `modules.file.format` (defaults to `{artist} - {title}`).
If no song is playing, the file will be empty.

### `path`

Controls which path the application writes the song info into (relative or absolute path).
You must ensure the location exists, i.e. all folders in the path must exist.

Defaults to `curent_song.txt`.

### `format`

Controls the format of the written text.
Interpolations are wrapped inside `{` and `}`, if you want to output a `{`, use `{{`.
These are the supported interpolations:

| Interpolation     | Description                                                                                       |
| ----------------- | ------------------------------------------------------------------------------------------------- |
| `{title}`         | The song's title.                                                                                 |
| `{artist}`        | The song's artist.                                                                                |
| `{album-name?}`   | The song's album name (or empty string).                                                          |
| `{album-tracks?}` | The album's track count (or empty string).                                                        |
| `{track-number?}` | The number of this track on the album (or empty string).                                          |
| `{source}`        | The provider of the current song. For gsmtc: `gsmtc::<executable>`, for the extension: `browser`. |
| `{duration?}`     | The song's duration (e.g. `1m23s`) (or empty string).                                             |

Defaults to `{artist} - {title}`.
