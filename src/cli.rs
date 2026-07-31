//! Command line definition.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

/// LICENSE file generator for open source projects.
#[derive(Debug, Parser)]
#[command(
    name = "jlic",
    version,
    about,
    long_about = None,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// `new` arguments usable without naming the subcommand: `jlic mit -o LICENSE`.
    #[command(flatten)]
    pub new: NewArgs,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a license file (MIT into ./LICENSE by default)
    New(NewArgs),

    /// List the supported licenses
    List {
        /// Print machine-readable JSON
        #[cfg(feature = "json")]
        #[arg(long)]
        json: bool,
    },

    /// Print the raw license template without substituting anything
    Show {
        /// SPDX identifier or alias
        license: String,
    },

    /// Print the notice block for a source file header
    Notice {
        /// SPDX identifier or alias
        #[arg(default_value = "mit")]
        license: String,

        #[command(flatten)]
        holder: HolderArgs,
    },

    /// Generate a shell completion script
    Completions {
        /// bash, zsh, fish, elvish or powershell
        shell: Shell,
    },

    /// Print a man page (roff) to stdout
    Man,
}

/// Copyright holder details, shared by `new` and `notice`.
#[derive(Debug, Args, Clone)]
pub struct HolderArgs {
    /// Copyright holder (defaults to JLIC_NAME or git config user.name)
    #[arg(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// Contact email (defaults to JLIC_EMAIL or git config user.email)
    #[arg(short, long, value_name = "EMAIL")]
    pub email: Option<String>,

    /// Year or range, e.g. 2026 or 2020-2026 (defaults to the current year)
    #[arg(short, long, value_name = "YEAR")]
    pub year: Option<String>,

    /// Leave the email out even when one is found
    #[arg(long, conflicts_with = "email")]
    pub no_email: bool,
}

#[derive(Debug, Args, Clone)]
pub struct NewArgs {
    /// SPDX identifier or alias: mit, apache-2.0, gpl-3.0-or-later, bsd-3-clause, mpl-2.0, isc, wtfpl
    #[arg(default_value = "mit")]
    pub license: String,

    #[command(flatten)]
    pub holder: HolderArgs,

    /// Path of the file to create
    #[arg(short, long, value_name = "PATH", default_value = "LICENSE")]
    pub output: PathBuf,

    /// Print to stdout instead of writing a file
    #[arg(long, conflicts_with = "output")]
    pub stdout: bool,

    /// Overwrite an existing file
    #[arg(short, long)]
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn bare_invocation_defaults_to_mit_license_file() {
        let cli = Cli::parse_from(["jlic"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.new.license, "mit");
        assert_eq!(cli.new.output, PathBuf::from("LICENSE"));
    }

    #[test]
    fn license_can_be_given_without_subcommand() {
        let cli = Cli::parse_from(["jlic", "apache-2.0", "--stdout"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.new.license, "apache-2.0");
        assert!(cli.new.stdout);
    }

    #[test]
    fn subcommand_wins_over_positional_license() {
        let cli = Cli::parse_from(["jlic", "list"]);
        assert!(matches!(cli.command, Some(Command::List { .. })));
    }

    #[test]
    fn stdout_conflicts_with_output() {
        assert!(Cli::try_parse_from(["jlic", "mit", "--stdout", "-o", "LIC"]).is_err());
    }
}
