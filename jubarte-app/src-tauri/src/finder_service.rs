//! The macOS Services behind Finder's "Redline with Jubarte" and "Convert to
//! PDF with Jubarte" (`NSServices` in `Info.plist`). AppKit glue only: it runs inside a live AppKit app, so it is
//! exercised by hand (see README → "Finder") and left out of line coverage like
//! the StoreKit bridge. Everything it hands over goes through
//! [`crate::finder::deliver`], which is tested.

use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSPasteboard, NSPasteboardTypeFileURL, NSUpdateDynamicServices,
};
use objc2_foundation::{NSString, NSURL};
use tauri::AppHandle;

define_class!(
    // The object AppKit calls for both services. Each selector is an
    // `NSMessage` in Info.plist plus `:userData:error:`.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JubarteFinderService"]
    #[ivars = AppHandle]
    struct FinderService;

    unsafe impl NSObjectProtocol for FinderService {}

    impl FinderService {
        #[unsafe(method(redlineFiles:userData:error:))]
        fn redline_files(
            &self,
            pasteboard: &NSPasteboard,
            _user_data: Option<&NSString>,
            _error: *mut *mut NSString,
        ) {
            crate::finder::deliver(
                self.ivars(),
                file_paths(pasteboard),
                Some(crate::finder::Intent::Redline),
            );
        }

        #[unsafe(method(convertFiles:userData:error:))]
        fn convert_files(
            &self,
            pasteboard: &NSPasteboard,
            _user_data: Option<&NSString>,
            _error: *mut *mut NSString,
        ) {
            crate::finder::deliver(
                self.ivars(),
                file_paths(pasteboard),
                Some(crate::finder::Intent::Convert),
            );
        }
    }
);

/// The files on a service pasteboard, as paths. Finder writes file
/// *reference* URLs (`file:///.file/id=…`); `filePathURL` resolves them.
fn file_paths(pasteboard: &NSPasteboard) -> Vec<String> {
    let Some(items) = pasteboard.pasteboardItems() else {
        return Vec::new();
    };
    // SAFETY: an immutable AppKit constant.
    let file_url = unsafe { NSPasteboardTypeFileURL };
    items
        .iter()
        .filter_map(|item| item.stringForType(file_url))
        .filter_map(|s| NSURL::URLWithString(&s))
        .filter_map(|url| url.filePathURL())
        .filter_map(|url| url.path())
        .map(|path| path.to_string())
        .collect()
}

/// Become the provider of the Info.plist services. Call once, on the main
/// thread, at startup.
pub fn register(app: &AppHandle) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let provider = FinderService::alloc(mtm).set_ivars(app.clone());
    let provider: Retained<FinderService> = unsafe { msg_send![super(provider), init] };
    // SAFETY: the provider implements the selector Info.plist names.
    unsafe { NSApplication::sharedApplication(mtm).setServicesProvider(Some(&provider)) };
    // The provider lives as long as the app; AppKit may hold it weakly.
    std::mem::forget(provider);
    // Refresh the Services menu so a fresh install shows the item at once.
    NSUpdateDynamicServices();
}
