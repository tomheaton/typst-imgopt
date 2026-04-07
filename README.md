# typst-imgopt

`typst-imgopt` is a small Typst image optimisation package backed by a Rust WebAssembly plugin.
The Rust side handles format detection, resizing, and re-encoding.
The Typst side handles layout-aware width inference and exposes a few wrapper functions that can be dropped into a document.

The plugin interface is byte-based and pure, which fits Typst's plugin model.

## Requirements

- [Rust](https://rust-lang.org/) stable with Cargo
- the `wasm32-unknown-unknown` target
- [Typst CLI](https://github.com/typst/typst) if you want to compile the example documents

## Repository layout

- `src/lib.rs`: the WebAssembly plugin
- `typst/imgopt.typ`: the Typst wrapper module
- `typst/imgopt.wasm`: the built plugin binary loaded by the wrapper
- `examples/main.typ`: the optimised example document
- `examples/main-no-opt.typ`: the same document without the wrapper
- `scripts/build-wasm.sh`: builds the wasm target and copies it into `typst/`
- `scripts/compare-pdf-size.sh`: builds both example PDFs and prints the size difference

## Behaviour

- JPEG input is re-encoded with the requested `quality`.
- PNG input is re-encoded losslessly by default.
- With `lossy-png: true`, opaque PNGs can be converted to JPEG.
- SVG, PDF, and unrecognised input are passed through unchanged.
- Raster images can be downscaled from the resolved Typst layout width and `target-ppi`.
- If `strip-metadata: false` and no resize is needed, the original JPEG or PNG bytes are kept.
- If a same-size PNG re-encode would be larger, the original PNG bytes are kept.

## Build

Install the wasm target once:

```sh
rustup target add wasm32-unknown-unknown
```

Build the plugin and copy it into the Typst package directory:

```sh
./scripts/build-wasm.sh
```

CI runs the same basic checks on pushes and pull requests: formatting, clippy, tests, wasm build, and example compilation.

## Example documents

Build the optimised example:

```sh
./scripts/build-wasm.sh
typst compile --root . examples/main.typ examples/main.pdf
```

Compare the optimised and unoptimised versions:

```sh
./scripts/compare-pdf-size.sh
```

If you want to run the comparison steps manually:

```sh
./scripts/build-wasm.sh
typst compile --root . examples/main.typ examples/main.pdf
typst compile --root . examples/main-no-opt.typ examples/main-no-opt.pdf
wc -c examples/main.pdf examples/main-no-opt.pdf
```

## Typst API

Inside this repository, the examples import the wrapper like this:

```typst
#import "../typst/imgopt.typ": imgopt-image, imgopt-image-bytes
```

The wrapper exposes these public helpers:

- `imgopt-image-bytes(raw, ...)` for byte input
- `imgopt-image(source, ...)` for either a path or raw bytes
- `imgopt-image-auto-bytes(raw, ...)` as a convenience form with `width: 100`
- `imgopt-image-auto(source, ...)` as the same convenience form for a path or bytes
- `infer-max-width-px(width, container-width, target-ppi: 220)` if you want the raw pixel calculation

The optimisation-specific arguments are `quality`, `target-ppi`, `lossy-png`, and `strip-metadata`.
The wrapper also forwards `height`, `format`, `fit`, `alt`, `scaling`, and `icc` to Typst's built-in `image` function.

Bytes-first use:

```typst
#let raw = read("figures/photo.jpg", encoding: none)

#imgopt-image-bytes(
  raw,
  width: 80%,
  quality: 78,
  target-ppi: 220,
)
```

Path or bytes convenience wrapper:

```typst
#imgopt-image(
  "figures/photo.jpg",
  width: 80%,
  quality: 78,
  target-ppi: 220,
)
```

If you are wrapping this inside another reusable Typst package, prefer the bytes-first entry points and let the calling document read the file.

## Width inference

The wrapper computes the pixel ceiling during layout, using the width of the current container rather than a fixed page width.

The rough calculation is:

$$
\operatorname{max\_width\_px} = \left\lceil \frac{\operatorname{container\_width}}{1\,\mathrm{in}} \cdot \frac{\operatorname{width\_percent}}{100} \cdot \operatorname{target\_ppi} \right\rceil
$$

If `width` is left as `auto`, no pixel ceiling is applied.

## Notes

- Typst plugins must stay pure.
- Plugins cannot read project files directly; Typst has to pass bytes in.
- Build for `wasm32-unknown-unknown`, not WASI.

## Licence

This repository is licensed under MIT
See `LICENSE`.

## Credits

- `flower.jpg` from [WikiMedia](https://commons.wikimedia.org/wiki/File:Flower_in_Austria.jpg)
- `flower.png` converted from the above JPEG
