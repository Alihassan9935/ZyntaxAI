use crate::capabilities::InjectionBackend;
use crate::textio::{Chord, PlatformError};
use std::process::Command;
use std::time::Duration;

const KEY_LEFTCTRL: u16 = 29;
const KEY_V: u16 = 47;
const KEY_C: u16 = 46;

fn letter(chord: Chord) -> &'static str {
    match chord {
        Chord::Copy => "c",
        Chord::Paste => "v",
    }
}

fn key_code(chord: Chord) -> u16 {
    match chord {
        Chord::Copy => KEY_C,
        Chord::Paste => KEY_V,
    }
}

pub fn send_chord(backend: InjectionBackend, chord: Chord) -> Result<(), PlatformError> {
    match backend {
        InjectionBackend::Wtype => run("wtype", &["-M", "ctrl", "-k", letter(chord), "-m", "ctrl"]),
        InjectionBackend::Ydotool => {
            let ctrl = KEY_LEFTCTRL;
            let key = key_code(chord);

            let events = [
                format!("{ctrl}:1"),
                format!("{key}:1"),
                format!("{key}:0"),
                format!("{ctrl}:0"),
            ];
            let mut args = vec!["key"];
            args.extend(events.iter().map(String::as_str));
            run("ydotool", &args)
        }
        InjectionBackend::Native => Err(PlatformError::Injection(
            "native injection should not be routed through the Wayland helpers".to_owned(),
        )),
        InjectionBackend::None => Err(PlatformError::InjectionUnavailable),
    }
}

const HELPER_TIMEOUT: Duration = Duration::from_secs(3);

fn run(program: &str, args: &[&str]) -> Result<(), PlatformError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| {
            PlatformError::Injection(format!(
                "could not run {program}: {err}. Install it, or switch the output mode to \
                 Copy to clipboard."
            ))
        })?;

    let deadline = std::time::Instant::now() + HELPER_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    use std::io::Read;
                    let _ = pipe.read_to_string(&mut stderr);
                }
                return Err(PlatformError::Injection(format!(
                    "{program} exited with {status}: {}",
                    stderr.trim()
                )));
            }
            Ok(None) if std::time::Instant::now() >= deadline => {
                let _ = child.kill();
                return Err(PlatformError::Injection(format!(
                    "{program} did not respond. If you are using ydotool, check that ydotoold \
                     is running."
                )));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(err) => {
                return Err(PlatformError::Injection(format!(
                    "could not wait for {program}: {err}"
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chords_map_to_the_right_keys() {
        assert_eq!(letter(Chord::Copy), "c");
        assert_eq!(letter(Chord::Paste), "v");
        assert_eq!(key_code(Chord::Copy), KEY_C);
        assert_eq!(key_code(Chord::Paste), KEY_V);
    }

    #[test]
    fn no_backend_reports_unavailable_rather_than_trying() {
        assert!(matches!(
            send_chord(InjectionBackend::None, Chord::Paste),
            Err(PlatformError::InjectionUnavailable)
        ));
    }

    #[test]
    fn native_is_not_routed_through_the_helpers() {
        assert!(matches!(
            send_chord(InjectionBackend::Native, Chord::Paste),
            Err(PlatformError::Injection(_))
        ));
    }

    #[test]
    fn a_missing_helper_explains_itself() {
        let error =
            run("zyntax-definitely-not-a-real-binary", &["--version"]).expect_err("must fail");
        let message = error.to_string();
        assert!(message.contains("Copy to clipboard"), "got: {message}");
    }

    #[test]
    fn a_hanging_helper_times_out() {
        if which("sleep").is_none() {
            eprintln!("skipped: no `sleep` binary");
            return;
        }
        let started = std::time::Instant::now();
        let error = run("sleep", &["30"]).expect_err("must time out");

        assert!(
            started.elapsed() < HELPER_TIMEOUT * 2,
            "should not have waited for sleep"
        );
        assert!(error.to_string().contains("did not respond"));
    }

    fn which(name: &str) -> Option<std::path::PathBuf> {
        std::env::split_paths(&std::env::var_os("PATH")?)
            .map(|dir| dir.join(name))
            .find(|path| path.is_file())
    }
}
