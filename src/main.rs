use clap::Parser;
use colored::Colorize;
use error::CliError;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

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
        return Err(std::io::Error::new(
            io::ErrorKind::Unsupported,
            CliError::UnsupportedOS,
        ));
    } // else
    output = Command::new("find")
        .arg(root_dir)
        .arg("-name")
        .arg("*.asd")
        .output()?;

    if let Some(exit_code) = output.status.code() {
        if exit_code != 0 {
            return Err(std::io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8(output.stderr).unwrap_or("find command failed".to_owned()),
            ));
        }
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| std::io::Error::new(io::ErrorKind::InvalidData, e))?;
    let lines = stdout.lines().map(|l| l.to_owned()).collect();

    Ok(lines)
}

fn main() -> Result<(), CliError> {
    let args = CliArgs::parse();

    if args.force {
        println!("WARN: running in force mode\n");
    }

    if args.dry_run {
        println!("running with dry run. no files affected\n");
    }

    if args.purge && !args.force && !args.dry_run {
        println!("purge mode will delete files. continue? (Y/n) ");

        let stdin = io::stdin();
        let input = &mut String::new();
        let _ = stdin.read_line(input);
        let key = input[0..1].as_bytes()[0];

        if key != 121 && key != 89 && key != 10 {
            println!("aborted by user");
            std::process::exit(0);
        }
    }

    println!("root dir: {}", &args.root_node);

    let files = get_asd_files_paths(&args.root_node)?;
    let mut sizes = Vec::with_capacity(files.len());

    for file in files {
        if let Ok(meta) = fs::metadata(&file) {
            sizes.push(meta.size());
        } else {
            println!("{} {}", "could not read metadata for:".red(), file.cyan());
        }

        println!(
            "({})\t{}",
            size_to_human_readable(sizes[sizes.len() - 1] as u32),
            file.replace(&args.root_node, "")
        );
    }

    // let (files, sizes) = process_node(&args.root_node, &args);

    let total_size_bytes = sizes.iter().sum::<u64>();

    println!();
    println!("total asd files: {}", sizes.len());
    println!(
        "total asd files size: {}",
        size_to_human_readable(total_size_bytes as u32)
    );

    todo!("implement purge and dry-run");

    // if args.dry_run {
    //     println!();
    //     println!("files that would be removed:");
    //     for f in files {
    //         println!("{f}");
    //     }
    // }

    Ok(())
}
