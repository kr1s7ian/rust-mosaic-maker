# rust-mosaic-maker
Cli tool that converts any image to an image composed of images inside a folder. Currently supports only pieces of fixed size.
```
Usage: mosaic-maker [OPTIONS] <INPUT_PATH> <OUTPUT_PATH> <PIECES_FOLDER> <PIECE_SIZE>
Arguments:
  <INPUT_PATH>
  <OUTPUT_PATH>
  <PIECES_FOLDER>
  <PIECE_SIZE>

Options:
  -d, --dither
  -t, --transparent_pieces
  -h, --help                Print help
  ```

This is a Minecraft crafting table made of Minecraft blocks (no dithering)

![Minecraft crafting table made of minecraft blocks](https://i.imgur.com/yIA6lcX.png)

This is the David made of minecraft blocks (with dithering)

![David made of minecraft blocks](https://i.imgur.com/avfZbty.jpg)
