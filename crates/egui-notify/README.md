# egui-notify

Notifications for egui, *toasts*: a copy of
[egui-notify](https://github.com/ItsEthra/egui-notify) 0.24.0 by ItsEthra,
under its MIT licence (`LICENSE`), with a button of an action. The workspace
puts it in place of the crate of crates.io by `[patch.crates-io]`.

## Scope

- `Toasts::new()`, then `add(Toast)` or `info`, `success`, `warning`,
  `error`, `basic`; `show(context)` in every frame. As the original.
- `Toast::action(words)`: a button of an action before the cross. A press on
  it dismisses the toast, and `show` gives back its `ToastId` in that frame.
- `Toast::id()`: the `ToastId` `Toasts::add` gave it.
- Each toast takes the presses on it in an area of its own, so a press on a
  toast never reaches what lies under it. The button of the action carries
  its words as its name for a screen reader.

```rust
use std::time::Duration;
use egui_notify::Toasts;

# egui_notify::__run_test_ctx(|context| {
let mut toasts = Toasts::default();
let undo = toasts
    .basic("«Pattern» is in the trash")
    .action("Undo")
    .duration(Some(Duration::from_secs(5)))
    .id();
for pressed in toasts.show(context) {
    if pressed == undo {
        // put the pattern back
    }
}
# });
```

| A toast with an action |
| --- |
| ![A toast with the words and the button of an action][picture-action] |

[picture-action]: img/action.png

## What changed against 0.24.0

| Part | Change |
| --- | --- |
| `Toast` | `id`, `action`; `Toast::action`, `Toast::id`, `ToastId` |
| `Toasts::add` | gives each toast its `ToastId` |
| `Toasts::show` | returns the `ToastId`s whose action was pressed; an area for each toast takes its presses; the action is drawn and answered |
| the crate | `deny(missing_docs)`; `collapsible_if` allowed, the code of the original kept as it is |

## Boundaries

It knows egui only. What an action does is the caller's.

## Errors

None: it only draws and answers.
