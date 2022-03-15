# CurrentSong 2

![example screenshot](https://i.imgur.com/u6hepdX.png)

This project is a rewrite of the [**CurrentSong Overlay**](https://github.com/Nerixyz/current-song-overlay). The core is
written in Rust now. That doesn't change much, but now the project supports
Windows' [`GlobalSystemMediaTransportControls`](https://docs.microsoft.com/uwp/api/windows.media.control).

The goal of the project is to create a **simple yet powerful** overlay that displays the currently playing song. There
are a few unique features separating this project:

* **Near zero latency** ⏱ All modules are created with this in mind.
* **Displaying Progress** 💯 Progress is displayed where available.
* **Display Album Art** 🖼
* **Customizable** 🔧 The overlay is customizable through CSS (`theme.css`). Modules and the server can be configured in
  a `config.toml` file.

# Setup

## Windows

* Download the latest [`current-song2.exe` from the releases tab](https://github.com/Nerixyz/current-song2/releases)
  and place it in any (preferably empty) folder.
* Run `current-song2.exe` by just double-clicking.
* On the first run, it will ask you if you want to add the application to
  autostart: ![autostart dialog](https://i.imgur.com/bxCVaMG.png)
    * If you click **Yes**, then it will add the app to autostart and start CurrentSong2 regularly. You can remove it,
      by running the app from the command line: `current-song2.exe --remove-autostart`.
    * If you click **No**, then the app will start and remember your decision in `config.toml`.
* In OBS, add a new **Browser Source** with the url set to `http://localhost:48457` (width and height should be your resolution, probably 1920x1080).
* To get extended info from your browser, install the extension [for Chrome or Edge 📦](https://chrome.google.com/webstore/detail/currentsong/alanjgmjccmkkpmpejgdhaodfjlmcone) or [for Firefox 📦](https://addons.mozilla.org/firefox/addon/current-song-overlay/).


On the first run a `config.toml` file will be created. To configure the application further,
  see [Configuration](#configuration).

### Stopping CurrentSong2

CurrentSong2 runs in the background. To stop it, open _Task Manager_, go to the _Processes_ tab and sort by _Name_ (
default setting). Search for `current-song2.exe` in the _Background Processes_ and stop the process.

**Alternatively**: In the _Task Manager_, go to _Details_ and search for `current-song2.exe`.

### Autostart

To remove the application from autostart, run `current-song2.exe --remove-autostart` from a terminal.

Alternatively you can **disable** the autostart entry in the Task Manager (startup tab).

# Configuration

⚠ The config is loaded at the start of CurrentSong. So in order to apply the configuration, you need to **restart** the
application.

The configuration uses the [toml](https://toml.io) format.

The default configuration looks like this:

```toml
[modules.gsmtc]
enabled = true

[modules.gsmtc.filter]
mode = "Exclude"
items = ["chrome.exe", "msedge.exe", "firefox.exe"]

[server]
port = 48457
custom_theme_path = "theme.css"
```

## `no_autostart`

This flag controls if the application will try to add itself to autostart.

* If it's `true`, then it won't add itself to autostart. _This doesn't mean it will be removed_
* If it's `false` (default), then it **will** check the autostart and possibly add itself there. You can still disable
  the entry on the _Task Manager_'s _Autostart_ tab since this is independent of the actual registry entry.

## GSMTC (Global System Media Transport Controls, Windows)

GSMTC uses Windows' own media tracking to provide metadata. However, not every application emits metadata to this system
or only limited metadata (specifically browsers; that's why they're excluded by default).

### Filter

You can control which applications will be included in the search for metadata through `modules.gsmtc.filter`. There are
three modes: `Disabled`,`Include`, and `Exclude`:

* `Disabled` will disable all filters, and let everything pass the filters:

```toml
[modules.gsmtc.filter]
mode = "Disabled"
```

* `Include` will only include applications listed in `items`. ⚠ This list is **case-sensitive**. For example, only
  include Spotify:

```toml
[modules.gsmtc.filter]
mode = "Include"
items = ["Spotify.exe"] # ⚠ notice the capital 'S', the filter is case-sensitive
```

* `Exclude` will include everything, except applications listed in `items`. ⚠ This list is **case-sensitive**. For
  example, don't include firefox:

```toml
[modules.gsmtc.filter]
mode = "Exclude"
items = ["firefox.exe"]
```

💡 You can see the application name in the _Task Manager_ by right-clicking and selecting _Properties_.

### `is_enabled`

Controls whether the module should be enabled or not.

## Server

### `custom_theme_path`

Controls the path from which a CSS theme will be loaded, defaults to `theme.css`. This is indented, so that you can keep
multiple themes in the folder and switch between them.

### `port`

Controls the local port on which the server is listening, defaults to `48457`.

⚠ Since you cannot set the port on the browser extension, you need to keep the default value if you want to use the
extension.

# Theming

You can theme the overlay through a `theme.css` file (or a different filename specified in `custom_theme_path`).

Themes _don't_ require a restart of the app, you only need to reload the browser.

💡 To debug the theme it's best to open the overlay in your browser and use its dev-tools. Go to `http://localhost:48457` in your browser.

# Planned Features

See more in the [projects tab](https://github.com/Nerixyz/current-song2/projects/1).

* **Output to File**
* **Better OBS Integration**
