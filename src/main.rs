use std::{
    env,
    io::{Read, Write},
    process::Command,
};

fn check() -> Result<(), String> {
    if !Command::new("git")
        .arg("help")
        .output()
        .unwrap()
        .status
        .success()
    {
        return Err("Git is not installed!".to_string());
    }

    if !Command::new("gh").output().unwrap().status.success() {
        return Err("GH CLI is not installed!".to_string());
    }

    Ok(())
}

enum Visibility {
    Public,
    Private,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Private => write!(f, "private"),
        }
    }
}

struct Repository {
    name: String,
    visibility: Visibility,
}

impl Repository {
    pub fn new() -> Repository {
        Self {
            name: "".to_string(),
            visibility: Visibility::Private,
        }
    }

    pub fn create(&self) -> Result<(), String> {
        let git_command_output = Command::new("git")
            .arg("init")
            .output()
            .map_err(|e| format!("Unable to create or reinitialize git repository: {}", e))?;

        if !git_command_output.status.success() {
            let stderr = String::from_utf8_lossy(&git_command_output.stderr);
            return Err(format!("Git init failed: {}", stderr.trim()));
        }

        let program = "gh";
        let formatted_visibility = &format!("--{}", self.visibility);
        let arguments = vec![
            "repo",
            "create",
            &self.name,
            "--source=.",
            formatted_visibility,
        ];


        let mut program_command = Command::new(program);
        let create_command = program_command.args(arguments);

        let output = create_command.output().expect(&format!("Unable to create repository {}!", self.name));

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if !stderr.is_empty() {
                return Err(format!("Failed to create repository: {}", stderr));
            } else {
                return Err("Failed to create repository: Unknown error".to_string());
            }
        }

        let repository_url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        println!("Repostory {} with visibility {} created successfully! -> {}", self.name, self.visibility, repository_url);

        Ok(())
    }
}

fn main() -> Result<(), String> {
    check()?;

    let args: Vec<String> = env::args().collect();
    let stdin = std::io::stdin();

    let mut repository = Repository::new();
    repository.name = env::current_dir()
        .unwrap()
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();

    if args.contains(&"--name".to_string()) {
        // validate length is 3
        if args.len() < 3 {
            return Err(
                "Not enough arguments given for --name. Expected: --name {NAME_OF_REPOSITORY}"
                    .to_string(),
            );
        }

        let position = args.iter().position(|x| x == "--name").unwrap();

        if args[position + 1].starts_with("--") {
            return Err("Repository name should not start with --, did you fuck up the positioning of arguments?".to_string());
        }

        repository.name = args[position + 1].clone();
    }

    if args.contains(&"--public".to_string()) {
        repository.visibility = Visibility::Public;
    }

    if args.contains(&"--private".to_string()) {
        repository.visibility = Visibility::Private;
    }

    print!(
        "Are you sure you want to create a {} repository with name {} [N/y] ",
        repository.visibility, repository.name
    );
    std::io::stdout().flush().unwrap();

    let mut response = String::new();
    stdin.read_line(&mut response).unwrap();

    match response.to_ascii_lowercase().split_at(1).0 {
        "y" => (),
        _ => { print!("Repository has NOT been created, have a good day o3o"); return Ok(()) }
    }

    repository.create()
}
