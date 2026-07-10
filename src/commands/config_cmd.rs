use crate::app_config::{self, resolve_config_path};
use crate::cli::ConfigAction;
use crate::error::Result;
use crate::utils::print_json;

pub fn handle_config(action: ConfigAction, json: bool) -> Result<()> {
    match action {
        ConfigAction::Generate { path, force } => {
            let out = resolve_config_path(path.as_deref());
            let existed = out.exists();
            app_config::generate_default_config(&out, force)?;
            if json {
                #[derive(serde::Serialize)]
                struct Out {
                    path: String,
                    overwritten: bool,
                    force: bool,
                }
                print_json(&Out {
                    path: out.display().to_string(),
                    overwritten: existed,
                    force,
                })?;
            } else {
                if existed && force {
                    eprintln!("Overwrote existing config at {}", out.display());
                } else {
                    eprintln!("Wrote default config to {}", out.display());
                }
                println!("{}", out.display());
            }
            Ok(())
        }
        ConfigAction::Path { path } => {
            let resolved = resolve_config_path(path.as_deref());
            let exists = resolved.exists();
            if json {
                #[derive(serde::Serialize)]
                struct Out {
                    path: String,
                    exists: bool,
                }
                print_json(&Out {
                    path: resolved.display().to_string(),
                    exists,
                })?;
            } else {
                println!("{}", resolved.display());
                if exists {
                    eprintln!("(file exists)");
                } else {
                    eprintln!(
                        "(no file; runtime uses built-in defaults — run `repslog config generate`)"
                    );
                }
            }
            Ok(())
        }
    }
}
