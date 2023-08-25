# Theming

Current Song 2 will load a custom theme from the path configured in [`server.custom_theme_path`](../../Configuration.md#custom_theme_path) (default: `theme.css`). The path is relative to the executable.

The theme is loaded after the default theme, so all your rules will overwrite the default rules.

## Debugging

When creating your theme, it's best to use your browser to view the overlay. Simply navigate to [localhost:48457](http://localhost:48457){:target="\_blank"} and open your DevTools. To reload the theme, reload the page.

## Elements

```html
<body>
    <div id="song-container">
        <div id="image-container">
            <img id="image" />
        </div>
        <div id="song-info">
            <div id="mq-title">
                <!-- <marquee-related> -->
                <h1 id="title">Song Title</h1>
                <!-- </marquee-related> -->
            </div>
            <div id="mq-subtitle">
                <!-- <marquee-related> -->
                <h2 id="subtitle">Roog</h2>
                <!-- </marquee-related> -->
            </div>
        </div>
        <div id="progress"></div>
    </div>
</body>
```

### Attributes

Current Song 2 reflects query and hash parameters in the DOM through attributes on the `<html>` element. These can be used through [attribute selectors](https://developer.mozilla.org/docs/Web/CSS/Attribute_selectors) in CSS.
All query parameters are prefixed with `data-query-` and all hash parameters are prefixed with `data-hash-`:

```html title="http://localhost:48457/?align=left&color=red#animate=yes&shadow=none"
<html data-query-align="left" data-query-color="red" data-hash-animate="yes" data-hash-shadow="none">
    <head>
        <!-- ... -->
    </head>
    <body>
        <!-- ... -->
    </body>
</html>
```

## CSS Classes

The overlay will conditionally set a few CSS classes on elements by default.

### `song-container`

| Name               | Description                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------------- |
| `is-spotify`       | Set if the source is the Spotify app. This might be useful, because Spotify embeds its logo in the thumbnail. |
| `with-image`       | Set if the song has an image.                                                                                 |
| `with-progress`    | Set if progress information is available.                                                                     |
| `with-album`       | Set if the album name is known.                                                                               |
| `has-album-tracks` | Set if the amount of album tracks is known.                                                                   |
| `has-track`        | Set if the track number is known.                                                                             |
| `vanish`           | Set if the track is paused.                                                                                   |

### `image`

| Name      | Description                           |
| --------- | ------------------------------------- |
| `spotify` | Set if the source is the Spotify app. |

## CSS Variables

The overlay sets a few CSS variables on the `#song-container` element.

| Name             | Description                       |
| ---------------- | --------------------------------- |
| `--image-url`    | URL of the image to be displayed. |
| `--title`        | Title of the song.                |
| `--artist`       | Artist of the song.               |
| `--album`        | Album name.                       |
| `--album-tracks` | Amount of album tracks.           |
| `--track-number` | Number of the current track.      |

### Default Variables

The default theme exposes a few default variables for easier customization that you can overwrite e.g. in the `:root` or `#song-container` selector.

| Name                              | Description                                                      |
| --------------------------------- | ---------------------------------------------------------------- |
| `--theme-color`                   | The background color of the overlay.                             |
| `--text-color`                    | The text color on the overlay.                                   |
| `--font`                          | The font of the text on the overlay.                             |
| `--shadow-color`                  | The shadow color of the container.                               |
| `--rounded`                       | The amount of rounding applied to the edges of the container.    |
| `--max-width`                     | The maximum width of the container.                              |
| `--height`                        | The default height of the container.                             |
| `--max-height`                    | The maximum height of the container.                             |
| `--min-height`                    | The minimum height of the container.                             |
| `--container-shadow`              | The container shadow.                                            |
| `--progress-height`               | The height of the progress bar.                                  |
| `--progress-color`                | The color of the progress bar.                                   |
| `--progress-border-radius`        | The roundness of the progress bar.                               |
| `--progress-shadow`               | The shadow of the progress bar.                                  |
| `--use-marquee`                   | `true` if the text should scroll across if it's clipped.         |
| `--marquee-speed`                 | The speed of the scrolling.                                      |
| `--marquee-pause-duration`        | The duration in milliseconds to pause when an edge is reached.   |
| `--marquee-repeat-pause-duration` | The duration in milliseconds to pause when the start is reached. |
