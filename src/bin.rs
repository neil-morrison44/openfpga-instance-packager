use clap::Parser;
use instance_packager::{build_jsons_for_core, find_cores_with_package_json, PACKAGER_NAME};
use question::{Answer, Question};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    pocket_root_path: PathBuf,
    #[arg(short, long)]
    all: bool,
}

fn main() {
    let args = Args::parse();
    let path = args.pocket_root_path;
    let cores_list = find_cores_with_package_json(&path).unwrap();
    let core_count = cores_list.len();

    if cores_list.len() == 0 {
        println!("Found 0 cores with an {PACKAGER_NAME}, exiting...");
        return;
    }

    println!("Found {core_count} cores with an {PACKAGER_NAME}:\n");
    for (i, core_name) in cores_list.iter().enumerate() {
        println!("{}: {core_name}", i + 1);
    }
    println!("");

    let do_all_cores = || {
        for core_name in &cores_list {
            build_jsons_for_core(
                &path,
                &core_name,
                |file_name| {
                    println!("Wrote {}", file_name);
                },
                |file_name, message| {
                    println!("Skipped {file_name} \n {message}");
                },
            )
            .unwrap();
        }
    };

    if args.all {
        do_all_cores();
        return;
    }

    let numbers: Vec<String> = (1..=core_count).map(|i| i.to_string()).collect();
    let mut all_choices = vec!["all"];
    all_choices.extend(numbers.iter().map(|s| s.as_str()));

    if let Some(answer) = Question::new("Pick a core or all?")
        .acceptable(all_choices)
        .until_acceptable()
        .default(Answer::RESPONSE("all".to_string()))
        .show_defaults()
        .tries(2)
        .clarification("Enter \"all\" or the number listed of a core above")
        .ask()
    {
        match answer {
            Answer::RESPONSE(res) => match res.as_str() {
                "all" => {
                    do_all_cores();
                }
                _ => {
                    let index: usize = res.parse().unwrap();
                    let core_name = &cores_list[index - 1];
                    build_jsons_for_core(
                        &path,
                        &core_name,
                        |file_name| {
                            println!("Wrote {}", file_name);
                        },
                        |file_name, message| {
                            println!("Skipped {file_name} \n {message}");
                        },
                    )
                    .unwrap();
                }
            },
            _ => {}
        }
    }
}
