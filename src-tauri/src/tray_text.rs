//! Native AppKit bridge for two-line tray text rendering.
//!
//! Uses objc2 to set an NSAttributedString on the NSStatusBarButton,
//! enabling multi-line display in the macOS menu bar.

use objc2::runtime::AnyObject;
use objc2::AnyThread;
use objc2_app_kit::{
    NSFont, NSFontAttributeName, NSFontWeightLight, NSMutableParagraphStyle,
    NSParagraphStyleAttributeName, NSStatusBarButton,
};
use objc2_foundation::{MainThreadMarker, NSMutableAttributedString, NSRange, NSString, NSNumber};

/// Set a two-line attributed title on the tray icon's status bar button.
///
/// Creates an NSAttributedString with `"line1\nline2"`, 8pt light system font,
/// with negative baseline offset for vertical centering.
///
/// Must be called from the main thread (guaranteed by `with_inner_tray_icon`).
pub fn set_two_line_title(tray: &tray_icon::TrayIcon, line1: &str, line2: &str) {
    let Some(ns_status_item) = tray.ns_status_item() else {
        return;
    };
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };

    let text = format!("{}\n{}", line1, line2);
    let ns_text = NSString::from_str(&text);

    let attr_str =
        NSMutableAttributedString::initWithString(NSMutableAttributedString::alloc(), &ns_text);

    let full_range = NSRange::new(0, ns_text.len());

    // 8pt light system font for better centering in menu bar
    let font = unsafe { NSFont::systemFontOfSize_weight(8.0, NSFontWeightLight) };

    // Paragraph style with alignment centered
    let para_style = NSMutableParagraphStyle::new();
    para_style.setAlignment(objc2_app_kit::NSTextAlignment::Center);
    para_style.setLineSpacing(0.0);

    // Apply attributes to the full range
    // SAFETY: NSFont and NSMutableParagraphStyle are valid attribute value types.
    // Pointer casts are safe because both types inherit from NSObject -> AnyObject.
    unsafe {
        let font_obj: &AnyObject = &*(font.as_ref() as *const NSFont as *const AnyObject);
        let para_obj: &AnyObject =
            &*(para_style.as_ref() as *const NSMutableParagraphStyle as *const AnyObject);

        // NSNumber for baseline offset (negative moves up)
        let baseline_offset = NSNumber::new_i32(-1);
        let baseline_obj: &AnyObject = &*(baseline_offset.as_ref() as *const NSNumber as *const AnyObject);

        attr_str.addAttribute_value_range(NSFontAttributeName, font_obj, full_range);
        attr_str.addAttribute_value_range(NSParagraphStyleAttributeName, para_obj, full_range);

        // Split range: apply baseline offset to second line only
        let line1_len = line1.len();
        if line2.len() > 0 {
            let line2_range = NSRange::new(line1_len + 1, line2.len());
            attr_str.addAttribute_value_range(
                objc2_app_kit::NSBaselineOffsetAttributeName,
                baseline_obj,
                line2_range,
            );
        }
    }

    // Set on button
    let button: Option<objc2::rc::Retained<NSStatusBarButton>> =
        ns_status_item.button(mtm);
    if let Some(button) = button {
        button.setAttributedTitle(&attr_str);
    }
}

/// Reset to a plain title (single-line). Falls back to tray.set_title().
pub fn set_plain_title(tray: &tray_icon::TrayIcon, text: &str) {
    tray.set_title(Some(text));
}
