use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn collect_wgsl_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == "target" || name == ".git" {
                    continue;
                }
            }
            collect_wgsl_files(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("wgsl") {
            files.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_wgsl_files(Path::new("src"), &mut files)?;

    let mut had_error = false;
    for path in files {
        let source = fs::read_to_string(&path)?;
        if let Err(err) = naga::front::wgsl::parse_str(&source) {
            had_error = true;
            eprintln!(
                "WGSL error in {}:\n{}",
                path.display(),
                err.emit_to_string(&source)
            );
        }
    }

    if had_error {
        std::process::exit(1);
    }

    println!("WGSL check passed.");
    Ok(())
}
