#import "../typst/imgopt.typ": imgopt-image, infer-max-width-px
#let jpeg-source = read("assets/flower.jpg", encoding: none)
#let png-source = read("assets/flower.png", encoding: none)

#set page(width: 210mm, margin: (x: 25mm, y: 20mm))
#set text(font: "Libertinus Serif", 11pt)

= typst-imgopt demo

This document demonstrates the plugin wrapper with one JPEG example and one PNG example.

- The JPEG example uses 80% width at 220 ppi.
- The PNG example uses 55% width at 180 ppi.

#figure(
  imgopt-image(
    jpeg-source,
    width: 80%,
    quality: 78,
    target-ppi: 220,
  ),
  caption: [JPEG source with an inferred width bound at 80% and 220 ppi.],
)

#figure(
  imgopt-image(
    png-source,
    width: 55%,
    quality: 78,
    target-ppi: 180,
  ),
  caption: [PNG source with an inferred width bound at 55% and 180 ppi.],
)

The wrapper infers max widths of
#layout(size => [#infer-max-width-px(80%, size.width, target-ppi: 220) px])
for the JPEG figure and
#layout(size => [#infer-max-width-px(55%, size.width, target-ppi: 180) px])
for the PNG figure.
