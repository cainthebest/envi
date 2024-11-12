use {
    clap::Parser,
    rustc_version::{version, version_meta, Channel},
    std::{collections::HashMap, process::Command},
    sysinfo::{CpuExt, System, SystemExt},
    toml::Value,
    which::which,
};

#[derive(Parser)]
#[command(
    name = "envin",
    version = "0.0.2",
    about = "Displays information about your Rust environment."
)]
struct Args {
    /// Print system information
    #[arg(long)]
    system: bool,

    /// Print Rust-related information
    #[arg(long)]
    rust: bool,

    /// Print project information
    #[arg(long)]
    project: bool,

    /// Print all information
    #[arg(long)]
    all: bool,
}

fn main() {
    let args = Args::parse();

    let all = args.all || (!args.system && !args.rust && !args.project);

    if args.system || all {
        let system_info = SystemInfo::collect();
        system_info.display();
        println!();
    }

    if args.rust || all {
        let rust_info = RustInfo::collect();
        rust_info.display();
        println!();
    }

    if args.project || all {
        let project_info = ProjectInfo::collect();
        project_info.display();
        println!();
    }
}

struct SystemInfo {
    os: String,
    cpu: String,
    cpu_cores: usize,
    memory: String,
    shell: String,
}

impl SystemInfo {
    fn collect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let os_name = sys.name().unwrap_or_else(|| "Unknown".into());
        let os_version = sys.os_version().unwrap_or_else(|| "".into());
        let os = format!("{} {}", os_name, os_version);
        let cpu = sys.global_cpu_info().brand().to_string();
        let cpu_cores = sys.physical_core_count().unwrap_or(0);
        let memory = format!(
            "{:.2} GB / {:.2} GB",
            sys.used_memory() as f64 / 1024.0 / 1024.0,
            sys.total_memory() as f64 / 1024.0 / 1024.0
        );
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "Unknown".into());

        Self {
            os,
            cpu,
            cpu_cores,
            memory,
            shell,
        }
    }

    fn display(&self) {
        println!("{}System Information{}", "=".repeat(10), "=".repeat(10));
        println!("  OS     : {}", self.os);
        println!("  CPU    : {} ({} cores)", self.cpu, self.cpu_cores);
        println!("  Memory : {}", self.memory);
        println!("  Shell  : {}", self.shell);
    }
}

struct RustInfo {
    compiler: CompilerInfo,
    cargo: ToolInfo,
    rustup: ToolInfo,
}

impl RustInfo {
    fn collect() -> Self {
        Self {
            compiler: CompilerInfo::collect(),
            cargo: ToolInfo::collect("Cargo", "cargo"),
            rustup: ToolInfo::collect("Rustup", "rustup"),
        }
    }

    fn display(&self) {
        println!("{}Rust Information{}", "=".repeat(10), "=".repeat(10));
        self.compiler.display();
        self.cargo.display();
        self.rustup.display();
    }
}

struct CompilerInfo {
    version: String,
    host: String,
    release: String,
    commit_hash: String,
    commit_date: String,
    channel: String,
}

impl CompilerInfo {
    fn collect() -> Self {
        match version() {
            Ok(ver) => {
                let meta = version_meta().unwrap();
                Self {
                    version: ver.to_string(),
                    host: meta.host.clone(),
                    release: meta.short_version_string.clone(),
                    commit_hash: meta.commit_hash.clone().unwrap_or_else(|| "Unknown".into()),
                    commit_date: meta.commit_date.clone().unwrap_or_else(|| "Unknown".into()),
                    channel: match meta.channel {
                        Channel::Dev => "Dev".into(),
                        Channel::Nightly => "Nightly".into(),
                        Channel::Beta => "Beta".into(),
                        Channel::Stable => "Stable".into(),
                    },
                }
            }
            Err(_) => Self {
                version: "rustc not found.".into(),
                host: "Unknown".into(),
                release: "Unknown".into(),
                commit_hash: "Unknown".into(),
                commit_date: "Unknown".into(),
                channel: "Unknown".into(),
            },
        }
    }

    fn display(&self) {
        println!("  {}Compiler{}", "-".repeat(10), "-".repeat(10));
        println!("    Version     : {}", self.version);
        println!("    Host        : {}", self.host);
        println!("    Release     : {}", self.release);
        println!("    Commit Hash : {}", self.commit_hash);
        println!("    Commit Date : {}", self.commit_date);
        println!("    Channel     : {}", self.channel);
    }
}

struct ToolInfo {
    name: String,
    version: String,
}

impl ToolInfo {
    fn collect(name: &str, command: &str) -> Self {
        if which(command).is_ok() {
            let output = Command::new(command).arg("--version").output().ok();
            if let Some(output) = output {
                let version_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let version = extract_version_number(&version_info);
                Self {
                    name: name.into(),
                    version,
                }
            } else {
                Self {
                    name: name.into(),
                    version: format!("Failed to get {} version.", name),
                }
            }
        } else {
            Self {
                name: name.into(),
                version: format!("{} not found.", name),
            }
        }
    }

    fn display(&self) {
        println!("  {}{}", "-".repeat(10), "-".repeat(10));
        println!("    {} Version: {}", self.name, self.version);
    }
}

fn extract_version_number(info: &str) -> String {
    info.split_whitespace()
        .nth(1)
        .unwrap_or("Unknown")
        .to_string()
}

struct ProjectInfo {
    name: String,
    version: String,
    dependencies: Vec<DependencyInfo>,
}

impl ProjectInfo {
    fn collect() -> Self {
        let cargo_toml_path = std::path::Path::new("Cargo.toml");
        let cargo_lock_path = std::path::Path::new("Cargo.lock");

        let mut name = "Unknown".to_string();
        let mut version = "Unknown".to_string();
        let mut dependencies = Vec::new();

        if cargo_toml_path.exists() {
            if let Ok(cargo_toml_content) = std::fs::read_to_string(cargo_toml_path) {
                if let Ok(cargo_toml_value) = cargo_toml_content.parse::<Value>() {
                    let package = cargo_toml_value.get("package");
                    name = package
                        .and_then(|pkg| pkg.get("name"))
                        .and_then(|name| name.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    version = package
                        .and_then(|pkg| pkg.get("version"))
                        .and_then(|version| version.as_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let mut dep_versions = HashMap::new();

                    if cargo_lock_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(cargo_lock_path) {
                            if let Ok(cargo_lock_value) = content.parse::<Value>() {
                                if let Some(packages) =
                                    cargo_lock_value.get("package").and_then(|v| v.as_array())
                                {
                                    for package in packages {
                                        if let Some(name) =
                                            package.get("name").and_then(|v| v.as_str())
                                        {
                                            if let Some(version) =
                                                package.get("version").and_then(|v| v.as_str())
                                            {
                                                dep_versions
                                                    .insert(name.to_string(), version.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(deps) = cargo_toml_value
                        .get("dependencies")
                        .and_then(|v| v.as_table())
                    {
                        for (dep_name, value) in deps {
                            let specified_version = if let Some(version_str) = value.as_str() {
                                version_str.to_string()
                            } else if let Some(table) = value.as_table() {
                                if let Some(version_str) =
                                    table.get("version").and_then(|v| v.as_str())
                                {
                                    version_str.to_string()
                                } else {
                                    "Unknown".into()
                                }
                            } else {
                                "Unknown".into()
                            };

                            let resolved_version = dep_versions
                                .get(dep_name)
                                .cloned()
                                .unwrap_or_else(|| "Unknown".into());

                            dependencies.push(DependencyInfo {
                                name: dep_name.clone(),
                                specified_version,
                                resolved_version,
                            });
                        }
                    }
                }
            }
        }

        Self {
            name,
            version,
            dependencies,
        }
    }

    fn display(&self) {
        println!("{}Project Information{}", "=".repeat(10), "=".repeat(10));
        println!("  Name    : {}", self.name);
        println!("  Version : {}", self.version);

        if self.dependencies.is_empty() {
            println!("  Dependencies: None");
        } else {
            println!("  Dependencies:");
            for dep in &self.dependencies {
                println!(
                    "    - {} {} ({})",
                    dep.name, dep.specified_version, dep.resolved_version
                );
            }
        }
    }
}

struct DependencyInfo {
    name: String,
    specified_version: String,
    resolved_version: String,
}
