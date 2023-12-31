use spinoff::{spinners, Color, Spinner};
use std::io::{self, Read, Write};
use todo_git::{issue::Issue, todo_git::TodoGit, LATEST_ISSUE};
mod args;
use args::TodoGitCommand;

// TODO: Implement HTTP Response error handling
// TODO: Implement Multi todo-list issues handling

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clap method to parse command line arguments
    let command = TodoGitCommand::parse();

    match command {
        Some(TodoGitCommand::Edit) => {
            let mut sp = Spinner::new(spinners::Dots2, "Fetching latest issue", Color::Yellow);
            let client = reqwest::Client::new();
            let issues = Issue::get_issues_list(&client).await?;

            let issue_number = issues.get(LATEST_ISSUE).expect("No issues found!");

            if let Some(body) = &issue_number.body {
                sp.update(
                    spinners::Dots2,
                    "Successfully fetched latest issue",
                    Color::Green,
                );
                let tmp_file: std::path::PathBuf = std::env::temp_dir().join("todo-git.md");
                let mut tmp_todofile = std::fs::File::create(&tmp_file)?;
                tmp_todofile.write_all(body.as_bytes())?;

                let editor = option_env!("EDITOR").unwrap_or("vim");

                sp.success(&format!("Opening issue body in {}", editor));

                let status = std::process::Command::new(editor)
                    .arg(&tmp_file)
                    .status()
                    .expect("Error opening editor!");

                if !status.success() {
                    sp.fail("Error in opening editor!");
                    std::process::exit(1);
                }
                let mut sp = Spinner::new(spinners::Dots2, "Updating issue body", Color::Yellow);

                let mut tmp_todofile = std::fs::File::open(tmp_file)?;
                let mut body = String::new();
                tmp_todofile.read_to_string(&mut body)?;

                let update_issue = issue_number.update_issue(&client, &body).await?;

                if !update_issue.status().is_success() {
                    sp.fail("Error occurred while updating issue!");
                    std::process::exit(1);
                }

                sp.success("Successfully updated issue body!");
            }
        }
        Some(TodoGitCommand::View) => {
            let mut sp = Spinner::new(spinners::Dots2, "Fetching latest issue", Color::Yellow);
            let client = reqwest::Client::new();
            let issues = Issue::get_issues_list(&client).await?;

            let issue_number = issues.get(LATEST_ISSUE).expect("No issues found!");

            if let Some(body) = &issue_number.body {
                // View body in `glow` pager if it is installed
                if todo_git::find_command("glow").is_some() {
                    sp.update(
                        spinners::Dots2,
                        "Successfully fetched latest issue",
                        Color::Green,
                    );
                    let tmp_file: std::path::PathBuf = std::env::temp_dir().join("todo-git.md");

                    let mut tmp_todofile = std::fs::File::create(&tmp_file)?;
                    tmp_todofile.write_all(body.as_bytes())?;

                    let status = std::process::Command::new("glow")
                        .arg(&tmp_file)
                        .status()
                        .expect("Error opening editor!");

                    if !status.success() {
                        sp.fail("Error in opening editor!");
                        std::process::exit(1);
                    }
                } else {
                    println!("{body}");
                }
                sp.success("Successfully printed the issue content");
            } else {
                sp.success("Empty issue body");
            }
        }

        Some(TodoGitCommand::Delete) => {
            let mut sp = Spinner::new(spinners::Dots2, "Deleting latest issue", Color::Yellow);
            let client = reqwest::Client::new();
            let issues = Issue::get_issues_list(&client).await?;

            let issue_number = issues.get(LATEST_ISSUE).expect("No issues found!");

            let response = issue_number.delete_issue(&client).await?;

            if !response.status().is_success() {
                sp.fail(&format!("Failed to delete issue!: {}", response.status()));
            }

            sp.success(&format!(
                "Issue {} deleted from Github repo",
                issue_number.number.unwrap()
            ));
        }

        Some(TodoGitCommand::CreateConfig(interactive)) => {
            let mut sp = Spinner::new(spinners::Dots2, "Creating config file", Color::Yellow);
            let mut config = TodoGit::default();
            let todo_config = format!("{}/.{}.json", env!("HOME"), env!("CARGO_PKG_NAME"),);
            let config_path = std::path::PathBuf::from(&todo_config);

            if config_path.exists() {
                sp.fail(&format!(
                    "An config file already exists at {}",
                    config_path.display()
                ));
                std::process::exit(1);
            }

            if interactive {
                sp.stop();
                let mut owner = String::new();
                let mut repo = String::new();
                let mut token = String::new();

                println!("Enter Github owner: ");
                io::stdin()
                    .read_line(&mut owner)
                    .expect("Failed to read owner");
                println!("Enter Github repository: ");
                io::stdin()
                    .read_line(&mut repo)
                    .expect("Failed to read repo");
                println!("Enter Owner's Github token:");
                io::stdin()
                    .read_line(&mut token)
                    .expect("Failed to read token");
                println!();

                // trim the whitespace from the input
                config.owner = owner.trim().to_string();
                config.repo = repo.trim().to_string();
                config.token = token.trim().to_string();
            } else {
                sp.success(&format!("Edit the {} file", config_path.display()));
            }
            let mut sp = Spinner::new(spinners::Dots2, "Saving config file", Color::Yellow);

            let config = serde_json::to_string_pretty(&config).expect("Failed to create config");

            std::fs::write(&config_path, config)?;

            sp.success("Successfully created config file");
        }

        Some(TodoGitCommand::New { title, body }) => {
            let mut sp = Spinner::new(spinners::Dots2, "Creating new issue", Color::Yellow);
            let client = reqwest::Client::new();

            let new_issue = Issue::create_new(&client, title, body).await?;

            if !new_issue.status().is_success() {
                sp.warn(&format!(
                    "Failed to create new issue: {}",
                    new_issue.status()
                ));
                std::process::exit(1);
            }

            sp.success("Successfully created new issue");
        }

        Some(TodoGitCommand::Help) => {}

        None => {
            println!("Invalid command");
        }
    }

    Ok(())
}
