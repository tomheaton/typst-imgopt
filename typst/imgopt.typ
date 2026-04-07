#let plugin-mod = plugin("imgopt.wasm")

#let default-options = (
  quality: 82,
  lossy-png: false,
  strip-metadata: true,
)

#let optimise-bytes(raw, opts) = {
  let merged = default-options + opts
  let encoded = cbor.encode(merged)
  plugin-mod.optimise(raw, encoded)
}

#let normalise_width_percent(width) = {
  if width == auto {
    100
  } else if type(width) == int or type(width) == float or type(width) == decimal {
    calc.clamp(width, 1, 100)
  } else {
    calc.clamp(width / 1%, 1, 100)
  }
}

#let infer-max-width-px(
  width,
  container-width,
  target-ppi: 220,
) = {
  let clamped = normalise_width_percent(width)
  let target-width = clamped / 100 * container-width
  int(calc.ceil(target-width / 1in * target-ppi))
}

#let imgopt-image-bytes(
  raw,
  width: auto,
  height: auto,
  quality: 82,
  lossy-png: false,
  strip-metadata: true,
  target-ppi: 220,
  format: auto,
  fit: "cover",
  alt: none,
  scaling: auto,
  icc: auto,
) = {
  layout(size => {
    let clamped = normalise_width_percent(width)
    let resolved-width = if width == auto { auto } else { clamped * 1% }
    let opts = (
      quality: quality,
      max-width-px: if width == auto {
        none
      } else {
        infer-max-width-px(clamped, size.width, target-ppi: target-ppi)
      },
      lossy-png: lossy-png,
      strip-metadata: strip-metadata,
    )
    let out = optimise-bytes(raw, opts)

    image(
      out,
      format: format,
      width: resolved-width,
      height: height,
      fit: fit,
      alt: alt,
      scaling: scaling,
      icc: icc,
    )
  })
}

#let imgopt-image(
  source,
  width: auto,
  quality: 82,
  target-ppi: 220,
  lossy-png: false,
  strip-metadata: true,
  height: auto,
  format: auto,
  fit: "cover",
  alt: none,
  scaling: auto,
  icc: auto,
) = {
  let raw = if type(source) == bytes {
    source
  } else {
    read(source, encoding: none)
  }

  imgopt-image-bytes(
    raw,
    width: width,
    height: height,
    quality: quality,
    lossy-png: lossy-png,
    strip-metadata: strip-metadata,
    target-ppi: target-ppi,
    format: format,
    fit: fit,
    alt: alt,
    scaling: scaling,
    icc: icc,
  )
}

#let imgopt-image-auto-bytes(
  raw,
  width: 100,
  quality: 82,
  target-ppi: 220,
  lossy-png: false,
  strip-metadata: true,
) = {
  imgopt-image(
    raw,
    width: width,
    quality: quality,
    target-ppi: target-ppi,
    lossy-png: lossy-png,
    strip-metadata: strip-metadata,
  )
}

#let imgopt-image-auto(
  source,
  width: 100,
  quality: 82,
  target-ppi: 220,
  lossy-png: false,
  strip-metadata: true,
) = {
  imgopt-image(
    source,
    width: width,
    quality: quality,
    target-ppi: target-ppi,
    lossy-png: lossy-png,
    strip-metadata: strip-metadata,
  )
}
