use clap::Parser;
use colored::Colorize;
use error::CliError;
use std::fs;
use std::io;
use std::process::Command;

#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

pub mod error;

#[derive(Parser)]
struct CliArgs {
    root_node: String,
    #[arg(short, long, default_value("false"))]
    no_list: bool,
    #[arg(long, default_value("false"))]
    purge: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(short, long)]
    force: bool,
}

fn size_to_human_readable(size: u32) -> String {
    let size_float: f64 = size.into();
    let power = size_float.ln() / 1024f64.ln();
    let label_index = power as u64 | 0;
    let value = size_float / 1024f64.powi(label_index as i32);

    let label = match label_index {
        0 => String::from("Bytes"),
        _ => {
            let mut l =
                String::from(['K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'][label_index as usize - 1]);
            l.push_str("iB");
            l
        }
    };

    format!("{:.1} {}", value, label)
}

fn get_asd_files_paths(root_dir: &str) -> Result<Vec<String>, std::io::Error> {
    let output;

    if cfg!(target_os = "windows") {
        let mut path = String::from(root_dir);
        path.extend("\\*.asd".chars());

        let mut cmd = String::from("dir ");
        cmd.extend(path.chars());
        cmd.extend(" /o /s /b".chars());

        output = Command::new("cmd").args(&["/C", &cmd]).output()?;
    } else {
        output = Command::new("find")
            .args(&[root_dir, "-name", "*.asd"])
            .output()?;
    }

    if let Some(exit_code) = output.status.code() {
        if exit_code != 0 {
            return Err(std::io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ));
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().map(|l| l.to_owned()).collect();

    Ok(lines)
}

fn main() -> Result<(), CliError> {
    let args = CliArgs::parse();

    if args.force {
        println!("🚨 running in {} mode\n", "FORCE".red());
    }

    if args.dry_run {
        println!("running with dry run. no files affected\n");
    }

    if args.purge && !args.force && !args.dry_run {
        println!(
            "⚠️ purge mode will delete files. {}? (Y/n) ",
            "continue".yellow()
        );

        let stdin = io::stdin();
        let input = &mut String::new();
        let _ = stdin.read_line(input);
        let key = input[0..1].as_bytes()[0];

        if key != 121 && key != 89 && key != 10 {
            println!("aborted by user");
            std::process::exit(0);
        }
    }

    println!("🔍 searching into: {}", &args.root_node.cyan());

    let files = get_asd_files_paths(&args.root_node)?;
    let mut sizes = Vec::with_capacity(files.len());

    if files.len() == 0 {
        println!("no files found");
        return Ok(());
    }

    for file in files.iter() {
        if let Ok(meta) = fs::metadata(file) {
            #[cfg(target_os = "windows")]
            sizes.push(meta.file_size());

            #[cfg(target_os = "linux")]
            sizes.push(meta.size());
        } else {
            sizes.push(0);
            println!("{} {}", "could not read metadata for:".red(), file);
        }

        if !&args.no_list {
            println!(
                "({})\t{}",
                size_to_human_readable(sizes[sizes.len() - 1] as u32),
                file.replace(&args.root_node, "")
            );
        }
    }

    if !&args.no_list {
        println!();
    }

    let total_size_bytes = sizes.iter().sum::<u64>();

    println!("🗃️ total asd files: {}", sizes.len().to_string().yellow());
    println!(
        "🗄️ total asd files size: {} {}",
        size_to_human_readable(total_size_bytes as u32)
            .to_string()
            .yellow(),
        if args.purge {
            "(deleted)".red()
        } else {
            "".white()
        }
    );

    // todo!("implement purge and dry-run");

    if args.purge && args.dry_run {
        println!();
        println!("files that would be {}:", "removed".red());
        for f in files.iter() {
            println!("{f}");
        }
    }

    Ok(())
}
