# typst-imgopt

A Typst image optimisation package built as:

- a Rust WebAssembly plugin (`src/lib.rs`)
- a Typst wrapper module (`typst/imgopt.typ`)
- an example Typst document (`examples/main.typ`)

The plugin interface is byte-based and pure, matching Typst's plugin model.

## Requirements

- [Rust](https://rust-lang.org/) stable with Cargo
- The `wasm32-unknown-unknown` Rust target
- [Typst CLI](https://github.com/typst/typst) for compiling the example documents

## What it does

- Detects input format from bytes.
- Passes through SVG and PDF unchanged.
- Re-encodes JPEG with configurable quality.
- Re-encodes PNG losslessly by default.
- Optionally converts PNG to JPEG when `lossy_png` is enabled and no alpha channel exists.
- Downscales raster images from inferred layout width and `target-ppi`.
- Supports metadata stripping through re-encoding.

## Project structure

```sh
.
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── examples
│   ├── assets
│   │   ├── flower.jpg
│   │   └── flower.png
│   ├── main-no-opt.typ
│   └── main.typ
├── scripts
│   ├── build-wasm.sh
│   └── compare-pdf-size.sh
├── src
│   └── lib.rs
└── typst
    ├── imgopt.typ
    └── imgopt.wasm
```

## Build the plugin

1. Install the target once:

   ```sh
   rustup target add wasm32-unknown-unknown
   ```

2. Build and copy the wasm binary into the Typst package folder:

   ```sh
   ./scripts/build-wasm.sh
   ```

This writes `typst/imgopt.wasm`, which is loaded by `typst/imgopt.typ`.

The GitHub Actions workflow runs the same formatting, linting, test, wasm build,
and example compilation steps on every push and pull request.

## Compile the example document

```sh
./scripts/build-wasm.sh
typst compile examples/main.typ examples/main.pdf
```

## Compare PDF sizes (optimised vs unoptimised)

The repository includes two equivalent example papers:

- `examples/main.typ` (optimised through the plugin wrapper)
- `examples/main-no-opt.typ` (direct image embeds, no optimisation)

Build and compare in one step:

```sh
./scripts/compare-pdf-size.sh
```

Manual build commands:

```sh
./scripts/build-wasm.sh
typst compile examples/main.typ examples/main.pdf
typst compile examples/main-no-opt.typ examples/main-no-opt.pdf
wc -c examples/main.pdf examples/main-no-opt.pdf
```

## Typst API

Import:

```typst
#import "../typst/imgopt.typ": imgopt-image, imgopt-image-bytes
```

The package API is bytes-first.
Both public wrappers infer the raster width limit from the current layout width,
the requested `width`, and `target-ppi`.

Bytes-first usage:

```typst
#let raw = read("figures/photo.jpg", encoding: none)

#imgopt-image-bytes(
   raw,
  width: 80%,
  quality: 78,
   target-ppi: 220,
)
```

Path convenience wrapper:

```typst
#imgopt-image(
   "figures/photo.jpg",
   width: 80%,
   quality: 78,
   target-ppi: 220,
)
```

## How width inference works

The wrapper computes the pixel ceiling during layout using the current container width.

The useful estimate is:

$$
\text{max\_width\_px} = \left\lceil \text{document\_width\_in} \times \text{target\_ppi} \times \frac{\text{width\_percent}}{100} \right\rceil
$$

Notes:

- The plugin itself cannot inspect Typst layout context, so the Typst wrapper performs the inference before calling into WebAssembly.
- The result tracks the resolved container width at the call site, so it adapts to columns, grids, and other nested layout contexts.
- If `width` is left as `auto`, no pixel ceiling is applied.

## Notes

- Plugins must remain pure functions.
- Plugins cannot read project files directly; Typst must pass bytes in.
- Build for `wasm32-unknown-unknown` (not WASI).

## Licence

This repository is licensed under MIT. See `LICENSE`.

## Credits

- `flower.jpg` from [WikiMedia](https://commons.wikimedia.org/wiki/File:Flower_in_Austria.jpg)
- `flower.png` converted from the above JPEG
