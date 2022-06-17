# lishogi-gif - WIP

Fork of [lila-gif](https://github.com/niklasf/lila-gif) modified for shogi.

Webservice to render Gifs of shogi positions and games, and stream them
frame by frame.

![Example](/example.gif)

| size    | frames | colors | width   | height  |
| ------- | ------ | ------ | ------- | ------- |
| 844 KiB | 101    | 63     | 1188 px | 1092 px |

## Usage

```
lishogi-gif 0.1.0

USAGE:
    lishogi-gif [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --address <address>    Listen on this address [default: 127.0.0.1]
        --port <port>          Listen on this port [default: 6175]
```

## HTTP API

### `GET /image.gif`

![Game thumbnail](/image.gif)

```
curl http://localhost:6175/image.gif?sfen=lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL_b_20B2b_1 --output image.gif
```

| name        | type  | default                                   | description                                                 |
| ----------- | ----- | ----------------------------------------- | ----------------------------------------------------------- |
| **sfen**    | ascii | _starting position_                       | SFEN of the position.                                       |
| black       | utf-8 | _none_                                    | Name of the sente player. Limited to 100 bytes.             |
| white       | utf-8 | _none_                                    | Name of the gote player. Limited to 100 bytes.              |
| comment     | utf-8 | `https://github.com/WandererXII/lila-git` | Comment to be added to GIF meta data. Limited to 255 bytes. |
| lastMove    | ascii | _none_                                    | Last move in USI notation (like `7g7f`).                    |
| check       | ascii | _none_                                    | Square of king in check (like `5a`).                        |
| orientation |       | `black`                                   | Pass `white` to flip the board.                             |

### `POST /game.gif`

```javascript
{
  "white": "Molinari", // optional
  "black": "Bordais", // optional
  "comment": "lishogi.org", // optional
  "orientation": "sente", // default
  "delay": 75, // default frame delay in centiseconds
  "frames": [
    // [...]
    {
      "sfen": "lnsgkgsnl/1r5b1/pppppp+Bpp/6p2/9/2P6/PP1PPPPPP/7R1/LNSGKGSNL w - 4",
      "delay": 500, // optionally overwrite default delay
      "lastMove": "8h3c+", // optionally highlight last move
      "check": "5a" // optionally highlight king
    }
  ]
}
```

### `GET /example.gif`

```
curl http://localhost:6175/example.gif --output example.gif
```

## Technique

Instead of rendering vector graphics at runtime, all pieces are prerendered
on every possible background. This allows preparing a minimal color palette
ahead of time.

![Sprite](/theme/sprite.gif)

All thats left to do at runtime, is copying sprites and Gif encoding.
More than 95% of the rendering time is spent in LZW compression.

For animated games, frames only contain the changed squares on transparent
background. The example below is a frame from the animation.

![Example frame](/example-frame.gif)

## License

lishogi-gif is licensed under the GNU Affero General Public License, version 3 or
any later version, at your option.

The generated images include text in
[Noto Sans](https://fonts.google.com/specimen/Noto+Sans) (Apache License 2.0)
and a piece set by
[Colin M.L. Burnett](https://en.wikipedia.org/wiki/User:Cburnett)
(GFDL or BSD or GPL).
