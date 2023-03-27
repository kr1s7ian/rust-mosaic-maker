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
  -r, --recursive

  -d, --dither

  -t, --use_transparent_pieces

  -a, --algorithm <ALGORITHM>
          [default: kmeans] [possible values: kmeans, histogram]
  -i, --kmeans_iterations <KMEANS_ITERATIONS>
          [default: 10]
  -c, --kmeans_clusters <KMEANS_CLUSTERS>
          [default: 1]
  -s, --kmeans_min_score <KMEANS_MIN_SCORE>
          [default: 0]
  -h, --help
          Print help
```

This is a Minecraft crafting table made of Minecraft blocks (no dithering, and with a bit of playing around with kmeans cli options)

![Minecraft crafting table made of minecraft blocks](https://i.imgur.com/RRFyWcs.png)

This is the David made of minecraft blocks (with dithering)

![David made of minecraft blocks](https://i.imgur.com/xvYXZF5.jpg)
