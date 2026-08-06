use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("could not reach the system clipboard: {0}")]
    Unavailable(String),
    #[error("the clipboard does not contain text")]
    NotText,
}

pub struct Clipboard {
    inner: arboard::Clipboard,
}

impl Clipboard {
    pub fn new() -> Result<Self, ClipboardError> {
        Ok(Self {
            inner: arboard::Clipboard::new()
                .map_err(|err| ClipboardError::Unavailable(err.to_string()))?,
        })
    }

    pub fn text(&mut self) -> Result<String, ClipboardError> {
        match self.inner.get_text() {
            Ok(text) => Ok(text),
            Err(arboard::Error::ContentNotAvailable) => Err(ClipboardError::NotText),
            Err(err) => Err(ClipboardError::Unavailable(err.to_string())),
        }
    }

    pub fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.inner
            .set_text(text.to_owned())
            .map_err(|err| ClipboardError::Unavailable(err.to_string()))
    }

    pub fn primary_text(&mut self) -> Result<String, ClipboardError> {
        #[cfg(target_os = "linux")]
        {
            use arboard::{GetExtLinux, LinuxClipboardKind};
            match self
                .inner
                .get()
                .clipboard(LinuxClipboardKind::Primary)
                .text()
            {
                Ok(text) => Ok(text),
                Err(arboard::Error::ContentNotAvailable) => Err(ClipboardError::NotText),
                Err(err) => Err(ClipboardError::Unavailable(err.to_string())),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(ClipboardError::NotText)
        }
    }

    pub fn snapshot(&mut self) -> Option<String> {
        self.text().ok()
    }

    pub fn restore(&mut self, snapshot: Option<String>) {
        let Some(previous) = snapshot else { return };
        if let Err(err) = self.set_text(&previous) {
            tracing::warn!(%err, "could not restore the previous clipboard contents");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_behaviour_end_to_end() {
        let Ok(mut clipboard) = Clipboard::new() else {
            eprintln!("skipped: no display server available");
            return;
        };

        let users_clipboard = clipboard.snapshot();

        clipboard.set_text("zyntax clipboard test").expect("set");
        assert_eq!(clipboard.text().expect("get"), "zyntax clipboard test");

        clipboard
            .set_text("the user's own copied text")
            .expect("set");
        let snapshot = clipboard.snapshot();
        clipboard.set_text("scratch space").expect("set");
        clipboard.restore(snapshot);
        assert_eq!(
            clipboard.text().expect("get"),
            "the user's own copied text",
            "restoring a snapshot must return the previous contents"
        );

        clipboard.restore(None);
        assert_eq!(clipboard.text().expect("get"), "the user's own copied text");

        let _ = clipboard.primary_text();

        clipboard.restore(users_clipboard);
    }
}
