# Customization

Current Song 2 is highly customizable. You can make [custom themes](Theming/index.md) or [user scripts](User Scripts.md) to adjust the display to your liking.

## Query Parameters

The overlay supports a few query parameters that can be appended to the URL which alter the behavior of the overlay.
For example, [`http://localhost:48457/?pos=br`](http://localhost:48457/?pos=br){:target="\_blank"} will put the overlay in the bottom right corner. Here, `pos` is the query parameter and `br` is the value. You can chain multiple with `&` - for example [`http://localhost:48457/?pos=br`](http://localhost:48457/?pos=br&use-raw-data){:target="\_blank"} will additionally skip any cleanup done to the song info.

| Parameter         | Description                                                                                                           |
| ----------------- | --------------------------------------------------------------------------------------------------------------------- |
| `pos`, `position` | Position of the overlay. See [Theming/Position](Theming/index.md#position) for all valid values.                      |
| `use-raw-data`    | Skip any cleanup done to the song data (like removal of content in braces if text is too long). No value is required. |

## Hash Parameters

In addition to query parameters, the overlay supports parameters via the hash of a URL (everything after the `#`). Multiple parameters can be chained with `&`. For example, [`http://localhost:48457/#pos=br`](http://localhost:48457/#pos=br){:target="\_blank"} will put the overlay in the bottom right corner.

| Parameter         | Description                                                                                      |
| ----------------- | ------------------------------------------------------------------------------------------------ |
| `pos`, `position` | Position of the overlay. See [Theming/Position](Theming/index.md#position) for all valid values. |
