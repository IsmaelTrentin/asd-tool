use clap::Parser;
use std::fs;
use std::io;

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

fn process_node(node_path: &str, args: &CliArgs) -> (Vec<String>, Vec<u64>) {
    if let Ok(meta) = fs::metadata(node_path) {
        if meta.is_dir() == false {
            panic!("node_path must be a folder");
        }
    } else {
        panic!("path does not exist or is inaccessible");
    }

    // println!("processing node {node_path}");

    let mut asd_files: Vec<String> = Vec::with_capacity(10);
    let mut asd_sizes: Vec<u64> = Vec::with_capacity(10);

    let entries = fs::read_dir(node_path).expect("could not read {path}");
    for entry_result in entries {
        if let Ok(entry) = entry_result {
            let f_type = entry.file_type().unwrap();
            let path = entry.path().to_str().unwrap().to_string();

            if f_type.is_dir() {
                // println!("processing node {path}");
                // recurse
                let (files, sizes) = process_node(&path, args);

                if files.len() > 0 && !args.no_list {
                    println!(
                        "found\t{}\t({})\t{}",
                        files.len(),
                        size_to_human_readable(sizes.iter().sum::<u64>() as u32),
                        path.replace(&args.root_node, "")
                    );
                }

                asd_files.extend(files);
                asd_sizes.extend(sizes);
                continue;
            }

            if !path.ends_with(".asd") {
                continue;
            }

            // now is file and is a .asd file
            asd_files.push(path.clone());
            if let Ok(metadata) = entry.metadata() {
                asd_sizes.push(metadata.len());
                // print!("{}B\n", metadata.len());
            } else {
                asd_sizes.push(0);
                print!("failed to get size\n");
            }

            // delete asd
            if args.purge && !args.dry_run {
                match fs::remove_file(&path) {
                    Ok(_) => println!("removed {}", &path),
                    Err(err) => eprintln!("{err}"),
                }
            }
        }
    }

    (asd_files, asd_sizes)
}

fn main() {
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
    let (files, sizes) = process_node(&args.root_node, &args);

    let total_size_bytes = sizes.iter().sum::<u64>();

    println!();
    println!("total asd files: {}", files.len());
    println!(
        "total asd files size: {}",
        size_to_human_readable(total_size_bytes as u32)
    );

    if args.dry_run {
        println!();
        println!("files that would be removed:");
        for f in files {
            println!("{f}");
        }
    }
}
