use clap::Parser;

#[derive(Parser)]
#[command(name = "kd")]
#[command(about = "A crystal clear command-line dictionary.")]
#[command(version)]
pub struct Cli {
    /// Translate long query
    #[arg(short = 't', long)]
    pub text: bool,

    /// Don't use cached result
    #[arg(short = 'n', long)]
    pub nocache: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Choose color theme
    #[arg(short = 'T', long)]
    pub theme: Option<String>,

    /// Update offline dictionary
    #[arg(long)]
    pub update_dict: bool,

    /// Generate config sample
    #[arg(long)]
    pub generate_config: bool,

    /// Edit configuration file
    #[arg(long)]
    pub edit_config: bool,

    /// Show status
    #[arg(long)]
    pub status: bool,

    /// Query text
    #[arg(num_args = 1..)]
    pub query: Vec<String>,
}
