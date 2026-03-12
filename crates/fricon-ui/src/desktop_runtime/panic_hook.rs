use std::{any::Any, panic};

use rfd::{MessageButtons, MessageDialog, MessageLevel};

use crate::launch::InteractionMode;

pub(crate) fn install_panic_hook(interaction_mode: InteractionMode) {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        if interaction_mode != InteractionMode::Dialog {
            return;
        }
        let message = build_panic_dialog_message(panic_info.payload(), panic_info.location());
        let _ = panic::catch_unwind(|| {
            MessageDialog::new()
                .set_level(MessageLevel::Error)
                .set_title("Fricon crashed")
                .set_description(message)
                .set_buttons(MessageButtons::Ok)
                .show();
        });
    }));
}

pub(crate) fn build_panic_dialog_message(
    payload: &(dyn Any + Send),
    location: Option<&panic::Location<'_>>,
) -> String {
    let reason = if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    };
    let where_text = location.map_or_else(
        || "location unavailable".to_string(),
        |loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()),
    );
    format!("An unexpected internal error occurred.\n\nReason: {reason}\nLocation: {where_text}")
}
