# obusina-button

A button of a fixed size for egui, its label in the middle.

## Scope

- `SquareButton::new(label, size)`: text or an icon in the middle of a
  button of `size`, both ways, whatever the size of the label.
- `.selected(true)`: drawn as selected; a filled button is outlined instead.
- `.fill(colour)`: filled with a colour of its own; the label is black or
  white, whichever reads on it (`contrast`).
- `.text_size(points)`: the size of the label.
- `.name(text)`: what a screen reader reads, when the label says nothing,
  such as an icon.
- In a disabled `Ui` it is drawn faded and gives no clicks.

![Plain, selected, disabled, an icon, light, dark and selected fills][picture-kinds]

[picture-kinds]: img/kinds.png

## Boundaries

It knows egui and nothing of the application. It holds no text of its own.

## Errors

None: it only draws.
