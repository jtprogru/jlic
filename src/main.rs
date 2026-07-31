use std::io::{self, Write};

use anyhow::{Context as _, Result};
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use jlic::cli::{Cli, Command, HolderArgs, NewArgs};
use jlic::{Context, License, render_license, render_notice};

fn main() {
    if let Err(err) = run() {
        eprintln!("jlic: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => new(cli.new),
        Some(Command::New(args)) => new(args),
        #[cfg(feature = "json")]
        Some(Command::List { json }) => list(json),
        #[cfg(not(feature = "json"))]
        Some(Command::List {}) => list(),
        Some(Command::Show { license }) => show(&license),
        Some(Command::Notice { license, holder }) => notice(&license, &holder),
        Some(Command::Completions { shell }) => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
            Ok(())
        }
        Some(Command::Man) => {
            clap_mangen::Man::new(Cli::command())
                .render(&mut io::stdout())
                .context("cannot generate the man page")?;
            Ok(())
        }
    }
}

fn new(args: NewArgs) -> Result<()> {
    let license: License = args.license.parse()?;
    let ctx = context(&args.holder)?;

    warn_on_ignored_fields(license, &args.holder);

    let text = render_license(license, &ctx)?;

    if args.stdout {
        io::stdout()
            .write_all(text.as_bytes())
            .context("cannot write to stdout")?;
        return Ok(());
    }

    jlic::output::write(&args.output, &text, args.force)?;
    eprintln!("{} → {}", license.spdx_id(), args.output.display());
    Ok(())
}

fn notice(license: &str, holder: &HolderArgs) -> Result<()> {
    let license: License = license.parse()?;
    let ctx = context(holder)?;
    print!("{}", render_notice(license, &ctx)?);
    Ok(())
}

fn show(license: &str) -> Result<()> {
    let license: License = license.parse()?;
    print!("{}", license.template());
    Ok(())
}

#[cfg(feature = "json")]
fn list(json: bool) -> Result<()> {
    if json {
        let items: Vec<_> = License::ALL
            .iter()
            .map(|l| {
                serde_json::json!({
                    "id": l.spdx_id(),
                    "title": l.title(),
                    "description": l.description(),
                    "url": l.url(),
                    "aliases": l.aliases(),
                    "placeholders": {
                        "year": l.fields().year,
                        "holder": l.fields().holder,
                    },
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    print_table();
    Ok(())
}

#[cfg(not(feature = "json"))]
fn list() -> Result<()> {
    print_table();
    Ok(())
}

fn print_table() {
    let width = License::ALL
        .iter()
        .map(|l| l.spdx_id().len())
        .max()
        .unwrap_or(0);

    for license in License::ALL {
        let mark = if *license == License::default() {
            "*"
        } else {
            " "
        };
        println!(
            "{mark} {:width$}  {}",
            license.spdx_id(),
            license.title(),
            width = width
        );
    }
    println!("\n* — default license");
}

fn context(args: &HolderArgs) -> Result<Context> {
    let ctx = Context::resolve(
        args.year.as_deref(),
        args.name.as_deref(),
        args.email.as_deref(),
    )?;
    Ok(if args.no_email {
        Context { email: None, ..ctx }
    } else {
        ctx
    })
}

/// Warns when the supplied details will not reach the license text.
fn warn_on_ignored_fields(license: License, args: &HolderArgs) {
    if !license.fields().is_empty() {
        return;
    }
    if args.name.is_some() || args.email.is_some() || args.year.is_some() {
        eprintln!(
            "jlic: the {} text has no copyright line, so the name and year are not substituted \
             into it; use `jlic notice {}` for source file headers",
            license.spdx_id(),
            license.spdx_id().to_ascii_lowercase()
        );
    }
}
