use aetheris::game_data;
use std::{env, path::Path, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Game data: {error}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let command = args.first().and_then(|a| a.to_str()).unwrap_or("");
    match (command, &args[usize::from(!args.is_empty())..]) {
        ("inspect", [directory]) => serde_json::to_writer_pretty(std::io::stdout().lock(), &game_data::inventory(Path::new(directory))?)?,
        ("discover", roots) => {
            let roots = if roots.is_empty() { game_data::discovery::default_roots() } else { roots.iter().map(Into::into).collect() };
            serde_json::to_writer_pretty(std::io::stdout().lock(), &game_data::discovery::installations(&roots))?;
        }
        _ => return Err("Usage: aetheris-game-data inspect <client-directory>\n       aetheris-game-data discover [search-root]...".into()),
    }
    Ok(())
}
