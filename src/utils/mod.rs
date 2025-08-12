pub(super) mod crypto;
pub(super) mod file;
pub(super) mod zerobyte;

use copypasta::{ClipboardContext, ClipboardProvider};


pub(super) fn copy_text_to_clipboard(text: String) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(text).unwrap();
    ctx.get_contents().unwrap();  // Not sure why I have to get_contents for this to work on KDE
}