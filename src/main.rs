use std::io::{Read, Write};
use todo_git::{issue::Issue, todo_git::TodoGit, LATEST_ISSUE};
mod args;
use std::io;

// TODO: Implement HTTP Response error handling
// TODO: Implement Multi todo-list issues handling

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clap method to parse command line arguments
    let cli = args::cli();

    match cli.subcommand() {
        Some(("edit", _argmatches)) => {
            let client = reqwest::Client::new();
            let issues = Issue::get_issues_list(&client).await?;

            let issue_number = issues.get(LATEST_ISSUE).expect("No issues found!");

            if let Some(body) = &issue_number.body {
                const TMP_FILE: &str = "/tmp/todo-git.md";
                let mut tmp_todofile = std::fs::File::create(TMP_FILE)?;
                tmp_todofile.write_all(body.as_bytes())?;

                let editor = option_env!("EDITOR").unwrap_or_else(|| "vim");

                let status = std::process::Command::new(editor)
                    .arg(TMP_FILE)
                    .status()
                    .expect("Error opening editor!");

                if !status.success() {
                    println!("Error in opening editor!");
                    std::process::exit(1);
                }

                let mut tmp_todofile = std::fs::File::open(TMP_FILE)?;
                let mut body = String::new();
                tmp_todofile.read_to_string(&mut body)?;

                let update_issue = issue_number.update_issue(&client, &body).await?;

                if !update_issue.status().is_success() {
                    println!("Error occurred while updating issue!");
                    std::process::exit(1);
                }
            }
        }
        Some(("view", _argmatches)) => {
            let client = reqwest::Client::new();
            let issues = Issue::get_issues_list(&client).await?;

            let issue_number = issues.get(LATEST_ISSUE).expect("No issues found!");

            if let Some(body) = &issue_number.body {
                println!("{body}");
            }
        }

        Some(("delete", _argmatches)) => {
            let client = reqwest::Client::new();
            let issues = Issue::get_issues_list(&client).await?;

            let issue_number = issues.get(LATEST_ISSUE).expect("No issues found!");

            let response = issue_number.delete_issue(&client).await?;

            if !response.status().is_success() {
                panic!("Failed to delete issue!: {}", response.status());
            }

            println!(
                "Issue {} deleted from Github repo",
                issue_number.number.unwrap()
            );
        }

        Some(("create-config", argmatches)) => {
            let mut config = TodoGit::default();
            let todo_config = format!("{}/.{}.json", env!("HOME"), env!("CARGO_PKG_NAME"),);
            let config_path = std::path::PathBuf::from(&todo_config);

            if config_path.exists() {
                println!("An config file already exists at {}", config_path.display());
                std::process::exit(1);
            }

            if argmatches.contains_id("interactive") {
                println!();
                print!("Enter Github owner: ");
                io::stdin()
                    .read_line(&mut config.owner)
                    .expect("Failed to read owner");
                print!("\nEnter Github repository: ");
                io::stdin()
                    .read_line(&mut config.repo)
                    .expect("Failed to read repo");
                print!("\nEnter Owner's Github token:");
                io::stdin()
                    .read_line(&mut config.token)
                    .expect("Failed to read token");
                println!();
            } else {
                println!("Edit the {} file", config_path.display());
            }

            let config = serde_json::to_string_pretty(&config).expect("Failed to create config");

            std::fs::write(&config_path, config)?;
        }

        Some(("new", argmatches)) => {
            let client = reqwest::Client::new();

            let mut title_arg: Option<&str> = None;
            let mut body_arg: Option<&str> = None;
            if argmatches.contains_id("title") {
                title_arg = argmatches.get_one::<String>("title").map(|s| s.as_str());

                println!("title: {}", title_arg.unwrap());
            }

            if argmatches.contains_id("body") {
                body_arg = argmatches.get_one::<String>("body").map(|s| s.as_str());

                println!("body: {}", body_arg.unwrap());
            }

            let new_issue = Issue::create_new(&client, title_arg, body_arg).await?;

            if !new_issue.status().is_success() {
                println!("Failed to create new issue: {}", new_issue.status());
                std::process::exit(1);
            }

            println!("Successfully created new issue");
        }

        _ => {
            /* INFO:
             *
             * ArgMatches can be used if we want to have more than two
             * command line arguments consecutively.
             *
             * ```
             * if _argmatches.contains_id("needle") {
             *  let needles_from_haystack = Vec<_> =
             *                              _argmatches
             *                                  .get_many::<String>("needle")
             *                                  .expect("contains_id")
             *                                  .map(|s| s.as_str())
             *                                  .collect();
             *
             *  let needles = needles_from_haystack.join(", ");
             * }
             * ```
             *
             */
        }
    }

    Ok(())
}
