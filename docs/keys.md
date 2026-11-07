# Keys

<!-- Written from the tables by `UPDATE_KEYS=1 cargo test -p obusina keys`. Not edited by hand. -->

The table of each layer stands beside its code. `src/keys.rs` says which layer is on top; only that layer acts on a key. Every key is a second way to an action (`RULE ui.touch`).

## Global keys

They act in every layer that lets them through.

No key.

## The list of patterns

The list is shown, no menu is open, no field holds the keys. A letter typed searches, as Ctrl+F does; that is text, not a key. The global keys act.

| Key | Action |
| --- | --- |
| Ctrl+F | Search by name |
| Right, Left, Down, Up | Walk the cards |
| Enter | Open the card: the pattern, a new one, or the import |
| Delete | Move the pattern to the trash |
| F2 | Rename |

## The trash

The trash is shown, no menu is open. The global keys act.

| Key | Action |
| --- | --- |
| Right, Left, Down, Up | Walk the cards |
| Enter | The menu of the pattern: restore or delete for good |

## The field

A pattern is open, nothing stands over the field, no field holds the keys. The global keys act.

| Key | Action |
| --- | --- |
| Ctrl+Shift+Z | Redo |
| Ctrl+Z | Undo |
| Ctrl+S | Save now |
| Escape | Back to the list of patterns |
| P | Open the window of the palettes |
| Plus, Equals | Next palette |
| Minus | Previous palette |
| 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 | Take the colour of that place of the palette shown; 0, the empty place |

## The corners of the picture

The corners of the picture under the pattern are edited; the buttons end it. The global keys act.

No key.

## The window of the palettes

The window of the palettes is open, its menu is not. The global keys act.

| Key | Action |
| --- | --- |
| Escape | Close the window |
| Plus, Equals | Next palette |
| Minus | Previous palette |

## A colour dragged

A colour is dragged in the window of the palettes. The global keys do not act.

No key.

## The menu of the window of the palettes

The menu of a place of the window of the palettes. Escape closes it at once, with a query typed or the picker holding the keys; the other keys are those of a menu. The global keys do not act.

| Key | Action |
| --- | --- |
| Escape | Close the menu |
| Down | The next entry |
| Up | The entry before |
| Right | Into the entry |
| Left | Out of the level, or clear the query |
| PageDown | Forward by the plates shown |
| PageUp | Back by the plates shown |
| Home | The first entry; with a query, the caret to its start |
| End | The last entry; with a query, the caret to its end |
| Backspace | The character before the caret; with Ctrl, the whole query |
| Enter | Choose the entry |
| Tab | Nothing; the keyboard stays with the menu |

## A menu

Any other menu: the settings, a colour, the picture, a card of the list. The keys are those of `plate-menu`. A menu opens with no entry selected: Down selects the first, Up the last, Enter does nothing. The global keys do not act.

| Key | Action |
| --- | --- |
| Escape | Clear the query; with none, close the menu |
| Down | The next entry |
| Up | The entry before |
| Right | Into the entry |
| Left | Out of the level, or clear the query |
| PageDown | Forward by the plates shown |
| PageUp | Back by the plates shown |
| Home | The first entry; with a query, the caret to its start |
| End | The last entry; with a query, the caret to its end |
| Backspace | The character before the caret; with Ctrl, the whole query |
| Enter | Choose the entry |
| Tab | Nothing; the keyboard stays with the menu |

## A text field

A text field holds the keys. It takes them all. The global keys do not act.

No key.

