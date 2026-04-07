#set page(width: 210mm, margin: (x: 25mm, y: 20mm))
#set text(font: "Libertinus Serif", 11pt)

= typst-imgopt demo

This document matches the optimised example with one JPEG example and one PNG example.

- The JPEG example uses 80% width.
- The PNG example uses 55% width.

#figure(
  image(
    "assets/flower.jpg",
    width: 80%,
  ),
  caption: [JPEG source at 80% width.],
)

#figure(
  image(
    "assets/flower.png",
    width: 55%,
  ),
  caption: [PNG source at 55% width.],
)

The matching inferred max widths are
#layout(size => [#int(calc.ceil((80% * size.width) / 1in * 220)) px])
for the JPEG figure and
#layout(size => [#int(calc.ceil((55% * size.width) / 1in * 180)) px])
for the PNG figure.
