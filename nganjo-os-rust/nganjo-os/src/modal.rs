// ══════════════════════════════════════════════════════════════════════════
//  modal.rs — typed modal dialogs (confirm / input / message)
//  Uses boxed closures so callers can capture state without lifetime pain.
// ══════════════════════════════════════════════════════════════════════════

use crate::app::App;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModalKind { Confirm, Input, Message }

pub struct Modal {
    pub kind:       ModalKind,
    pub title:      String,
    pub body:       String,     // confirm: question text; message: body text
    pub input:      String,     // live buffer for Input variant
    pub confirm_cb: Option<Box<dyn FnOnce(&mut App)>>,
    pub input_cb:   Option<Box<dyn FnOnce(&mut App, String)>>,
}

impl Modal {
    pub fn confirm(
        question: impl Into<String>,
        cb: impl FnOnce(&mut App) + 'static,
    ) -> Self {
        Self {
            kind:       ModalKind::Confirm,
            title:      " CONFIRM".into(),
            body:       question.into(),
            input:      String::new(),
            confirm_cb: Some(Box::new(cb)),
            input_cb:   None,
        }
    }

    pub fn input(
        prompt: impl Into<String>,
        default: impl Into<String>,
        cb: impl FnOnce(&mut App, String) + 'static,
    ) -> Self {
        Self {
            kind:       ModalKind::Input,
            title:      prompt.into(),
            body:       String::new(),
            input:      default.into(),
            confirm_cb: None,
            input_cb:   Some(Box::new(cb)),
        }
    }

    pub fn message(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            kind:       ModalKind::Message,
            title:      title.into(),
            body:       body.into(),
            input:      String::new(),
            confirm_cb: None,
            input_cb:   None,
        }
    }
}
