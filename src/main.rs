use octorust::{auth::Credentials, Client};
use clap::Parser;
use serde::Deserialize;
use std::env;
use std::fs;
use std::process;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct IssueConfig {
    title: String,
    body: Option<String>,
    labels: Option<Vec<String>>,
    assignee: Option<String>,
    assignees: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RepoConfig {
    owner: String,
    repo: String,
    issues: Vec<IssueConfig>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let token = match env::var("GITHUB_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Error: GITHUB_TOKEN environment variable not set");
            process::exit(1);
        }
    };

    let config_content = match fs::read_to_string(&args.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config file: {}", e);
            process::exit(1);
        }
    };

    let config: RepoConfig = match toml::from_str(&config_content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing TOML: {}", e);
            process::exit(1);
        }
    };

    let github = Client::new(
        "rust-issue-generator".to_string(),
        Credentials::Token(token),
    )
    .expect("Failed to create GitHub client");

    for issue in config.issues {
        let request = octorust::types::IssuesCreateRequest {
            title: octorust::types::TitleOneOf::String(issue.title),
            body: issue.body.unwrap_or_default(),
            labels: issue.labels.unwrap_or_default().into_iter()
                .map(|l| octorust::types::IssuesCreateRequestLabelsOneOf::String(l))
                .collect(),
            assignee: issue.assignee.unwrap_or_default(),
            assignees: issue.assignees.unwrap_or_default(),
            milestone: None,
        };

        match github
            .issues()
            .create(&config.owner, &config.repo, &request)
            .await
        {
            Ok(created) => println!(
                "Created issue #{}: {}",
                created.body.number, created.body.html_url
            ),
            Err(e) => eprintln!("Failed to create issue: {}", e),
        }
    }
}
