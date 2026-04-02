use std::{
    env,
    io::{IsTerminal, stderr, stdout},
};

use fricon_cli::clap::{Parser, error::ErrorKind};
use pyo3::{prelude::*, pyfunction};

fn ignore_python_sigint(py: Python<'_>) -> PyResult<()> {
    let signal = py.import("signal")?;
    let sigint = signal.getattr("SIGINT")?;
    let default_handler = signal.getattr("SIG_DFL")?;
    _ = signal.call_method1("signal", (sigint, default_handler))?;
    Ok(())
}

fn command_name_from_argv0(argv0: &std::ffi::OsStr) -> String {
    std::path::Path::new(argv0)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| "fricon".to_string(), ToString::to_string)
}

fn has_console_output() -> bool {
    stdout().is_terminal() || stderr().is_terminal()
}

fn parse_error_exit_code(kind: ErrorKind) -> i32 {
    match kind {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
        _ => 2,
    }
}

#[expect(clippy::print_stderr, reason = "Error messages for CLI tool")]
fn main_impl<T: Parser + fricon_cli::Main>(py: Python<'_>) -> i32 {
    if ignore_python_sigint(py).is_err() {
        eprintln!("Failed to reset python SIGINT handler.");
        return 1;
    }

    let argv = env::args_os().skip(1);
    let cli = T::parse_from(argv);
    match cli.main() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    }
}

/// Main CLI entry point that delegates to fricon-cli binary.
///
/// Returns:
///     Exit code.
#[pyfunction]
#[must_use]
pub(crate) fn main(py: Python<'_>) -> i32 {
    main_impl::<fricon_cli::Cli>(py)
}

/// GUI only CLI entry point.
///
/// Returns:
///     Exit code.
#[pyfunction]
#[must_use]
#[expect(clippy::print_stderr, reason = "Error messages for CLI tool")]
pub(crate) fn main_gui(py: Python<'_>) -> i32 {
    if ignore_python_sigint(py).is_err() {
        eprintln!("Failed to reset python SIGINT handler.");
        return 1;
    }

    let argv: Vec<_> = env::args_os().skip(1).collect();
    let command_name = argv.first().map_or_else(
        || "fricon-gui".to_string(),
        |arg| command_name_from_argv0(arg),
    );
    let cli_help = match fricon_cli::render_help_for_command::<fricon_cli::Gui>(&command_name) {
        Ok(help) => help,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };
    match fricon_cli::Gui::try_parse_from(argv) {
        Ok(cli) => match cli.main_with_help(command_name, cli_help) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Error: {e}");
                1
            }
        },
        Err(parse_error) => {
            if has_console_output() {
                let exit_code = parse_error_exit_code(parse_error.kind());
                eprint!("{parse_error}");
                exit_code
            } else {
                match fricon_cli::launch_gui_with_context(command_name, cli_help, None, false) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        1
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fricon_cli::clap::error::ErrorKind;

    use super::parse_error_exit_code;

    #[test]
    fn parse_help_and_version_return_success_exit_code() {
        assert_eq!(parse_error_exit_code(ErrorKind::DisplayHelp), 0);
        assert_eq!(parse_error_exit_code(ErrorKind::DisplayVersion), 0);
    }

    #[test]
    fn parse_failure_returns_error_exit_code() {
        assert_eq!(parse_error_exit_code(ErrorKind::MissingRequiredArgument), 2);
    }
}
