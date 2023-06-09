use std::path::PathBuf;
use std::process::Command;

use miette::{IntoDiagnostic as _, WrapErr as _};
use wca::{Args, Context, Props};

use crate::Result;

pub(crate) fn format(cx: Context, _args: Args, _props: Props) -> Result {
    let source = cx.get_ref::<PathBuf>();

    let status = Command::new("git")
        .args(["diff", "--exit-code"])
        .stdout(std::process::Stdio::null())
        .status()
        .into_diagnostic()?;

    status
        .exit_ok()
        .into_diagnostic()
        .context("Git index is dirty; not running formatter")?;

    match source {
        Some(path) => {
            for entry in jwalk::WalkDir::new(path) {
                let entry = entry.into_diagnostic()?;

                let path = entry.path();
                if path.is_dir() {
                    continue;
                }

                let source = std::fs::read_to_string(&path)
                    .into_diagnostic()
                    .with_context(|| format!("reading `{}`", path.display()))?;

                let contents = crate::formatter::format(&source);
                std::fs::write(&path, contents)
                    .into_diagnostic()
                    .with_context(|| format!("writing `{}`", path.display()))?;
            }
        }
        None => todo!("WTF??"),
    }

    Ok(())
}

pub(crate) fn with(cx: Context, args: Args, _props: Props) -> Result {
    let mut args = args.0.into_iter();
    wca::parse_args!(args, path: PathBuf);

    cx.insert(path);

    Ok(())
}

pub(crate) fn highlight(cx: Context, _args: Args, _props: Props) -> Result {
    let path = cx.get_ref::<PathBuf>().unwrap();

    let input = std::fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("reading `{}`", path.display()))?;

    let input = crate::highlight::highlight(&input);

    println!("{input}");

    Ok(())
}
