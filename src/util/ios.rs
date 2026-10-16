//! UIKit calls the iOS build needs that SDL2 doesn't wrap.

use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};

#[repr(C)]
#[derive(Clone, Copy)]
struct UIEdgeInsets {
    top: f64,
    left: f64,
    bottom: f64,
    right: f64,
}

/// Opens a URL with UIApplication: web links go to Safari, `shareddocuments://<path>` to the Files app.
pub fn open_url(url: &str) {
    const NS_UTF8_STRING_ENCODING: usize = 4;

    unsafe {
        let string: *mut Object = msg_send![class!(NSString), alloc];
        let string: *mut Object =
            msg_send![string, initWithBytes: url.as_ptr() length: url.len() encoding: NS_UTF8_STRING_ENCODING];
        let ns_url: *mut Object = msg_send![class!(NSURL), URLWithString: string];
        let _: () = msg_send![string, release];

        if ns_url.is_null() {
            log::warn!("Not a valid URL: {}", url);
            return;
        }

        let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
        let options: *mut Object = msg_send![class!(NSDictionary), dictionary];
        let _: () = msg_send![app, openURL: ns_url options: options completionHandler: std::ptr::null_mut::<Object>()];
    }
}

/// Safe-area insets of the game's window in points, as (left, top, right, bottom).
pub fn safe_area_insets() -> (f32, f32, f32, f32) {
    unsafe {
        let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
        let windows: *mut Object = msg_send![app, windows];
        let window: *mut Object = msg_send![windows, firstObject];

        if window.is_null() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let insets: UIEdgeInsets = msg_send![window, safeAreaInsets];
        (insets.left as f32, insets.top as f32, insets.right as f32, insets.bottom as f32)
    }
}
