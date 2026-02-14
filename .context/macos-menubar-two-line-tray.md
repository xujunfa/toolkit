# macOS Menu Bar Two-Line Tray Text

How to render multi-line attributed text in the macOS menu bar tray icon from a Tauri v2 + Rust app, using objc2 bindings.

## Problem

`TrayIcon::set_title()` only supports single-line plain text. macOS menu bar items can display multi-line text via `NSAttributedString` + `setAttributedTitle()` on the `NSStatusBarButton`, but Tauri doesn't expose this.

## Solution Architecture

```
Tauri TrayIcon
  └─ with_inner_tray_icon(closure)     // runs on main thread
       └─ tray_icon::TrayIcon
            └─ ns_status_item()         // returns Retained<NSStatusItem>
                 └─ button(mtm)         // returns NSStatusBarButton
                      └─ setAttributedTitle(NSAttributedString)
```

## Dependencies (Cargo.toml)

```toml
[target.'cfg(target_os = "macos")'.dependencies]
tray-icon = "0.21"   # direct dep for TrayIcon::ns_status_item()
objc2 = "0.6"
objc2-foundation = { version = "0.3", features = [
    "NSString", "NSDictionary", "NSAttributedString", "NSRange"
] }
objc2-app-kit = { version = "0.3", features = [
    "NSFont", "NSFontDescriptor", "NSButton",
    "NSStatusItem", "NSStatusBar", "NSStatusBarButton",
    "NSControl", "NSParagraphStyle", "NSAttributedString",
    "objc2-core-foundation"
] }
```

Key: `tray-icon` must be a **direct** dependency (same version Tauri uses internally) because `with_inner_tray_icon` passes `&tray_icon::TrayIcon` to the closure.

## Core Implementation (`tray_text.rs`)

```rust
use objc2::runtime::AnyObject;
use objc2::AnyThread;  // provides ::alloc() on NSMutableAttributedString
use objc2_app_kit::{
    NSFont, NSFontAttributeName, NSFontWeightLight,
    NSMutableParagraphStyle, NSParagraphStyleAttributeName,
    NSStatusBarButton,
};
use objc2_foundation::{
    MainThreadMarker, NSMutableAttributedString, NSRange, NSString,
};

pub fn set_two_line_title(tray: &tray_icon::TrayIcon, line1: &str, line2: &str) {
    let Some(ns_status_item) = tray.ns_status_item() else { return };
    let Some(mtm) = MainThreadMarker::new() else { return };

    let text = format!("{}\n{}", line1, line2);
    let ns_text = NSString::from_str(&text);
    let attr_str = NSMutableAttributedString::initWithString(
        NSMutableAttributedString::alloc(), &ns_text,
    );
    let full_range = NSRange::new(0, ns_text.len());

    // Font: 9pt system light
    let font = unsafe { NSFont::systemFontOfSize_weight(9.0, NSFontWeightLight) };

    // Paragraph style: vertically centered in the 22pt menu bar
    // 2 lines x 11pt = 22pt -> fills the bar height exactly
    let para_style = NSMutableParagraphStyle::new();
    para_style.setLineSpacing(0.0);
    para_style.setMinimumLineHeight(11.0);
    para_style.setMaximumLineHeight(11.0);

    // Apply attributes (pointer casts: NSFont/NSMutableParagraphStyle -> AnyObject)
    unsafe {
        let font_obj: &AnyObject = &*(font.as_ref() as *const NSFont as *const AnyObject);
        let para_obj: &AnyObject = &*(para_style.as_ref()
            as *const NSMutableParagraphStyle as *const AnyObject);
        attr_str.addAttribute_value_range(NSFontAttributeName, font_obj, full_range);
        attr_str.addAttribute_value_range(
            NSParagraphStyleAttributeName, para_obj, full_range,
        );
    }

    // Set on button
    let button: Option<objc2::rc::Retained<NSStatusBarButton>> =
        ns_status_item.button(mtm);
    if let Some(button) = button {
        button.setAttributedTitle(&attr_str);
    }
}
```

## Calling from Async Context

`with_inner_tray_icon` dispatches the closure to the main thread. The closure must be `Send + 'static`, so any data it captures (strings) must be owned.

```rust
fn update_tray_two_line(app: &AppHandle, items: &[QuotaItem]) {
    let lines = format_tray_lines(items);
    if let Some(tray) = app.tray_by_id("main-tray") {
        if lines.line2.is_empty() {
            let _ = tray.with_inner_tray_icon(move |inner| {
                tray_text::set_plain_title(inner, &lines.line1);
            });
        } else {
            let _ = tray.with_inner_tray_icon(move |inner| {
                tray_text::set_two_line_title(inner, &lines.line1, &lines.line2);
            });
        }
    }
}
```

## Vertical Centering

The macOS menu bar is **22pt** high. To vertically center two lines:

- `minimumLineHeight` = `maximumLineHeight` = **11pt** (22 / 2)
- `lineSpacing` = 0
- No `paragraphSpacingBefore` needed

If fine-tuning is needed, `NSBaselineOffsetAttributeName` can shift text up/down:

```rust
// Available but not currently used:
// objc2_app_kit::NSBaselineOffsetAttributeName
```

## objc2 Gotchas

| Issue | Fix |
|---|---|
| `NSMutableAttributedString::alloc()` not found | Import `use objc2::AnyThread;` (provides `alloc()`) |
| `ns_status_item.button(mtm)` type inference failure | Annotate: `let button: Option<Retained<NSStatusBarButton>> = ...` |
| `addAttribute_value_range` expects `&AnyObject` | Pointer cast: `&*(font.as_ref() as *const NSFont as *const AnyObject)` |
| `setLineSpacing` / `setMinimumLineHeight` | These are **not** `unsafe` in objc2-app-kit 0.3 despite the macro annotation |
| `NSFontWeightLight` requires feature | `objc2-app-kit` needs `"NSFontDescriptor"` + `"objc2-core-foundation"` features |
| `tray_icon::TrayIcon` unresolved | Must add `tray-icon` as direct dependency, not just via Tauri |

## File Layout

```
src-tauri/src/
  tray_text.rs          # AppKit bridge (set_two_line_title / set_plain_title)
  lib.rs                # mod tray_text;
  commands/zenmux.rs    # format_tray_lines() + update_tray_two_line() call sites
```
