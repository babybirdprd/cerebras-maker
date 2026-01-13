// Cerebras-MAKER: Main Entry Point
// Supports both GUI and CLI modes

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), not(feature = "cli")),
    windows_subsystem = "windows"
)]

use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for CLI mode
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => print_help(),
            "--version" | "-v" => print_version(),
            "run" => run_script(&args[2..]),
            "analyze" => analyze_workspace(&args[2..]),
            "snapshot" => create_snapshot(&args[2..]),
            "rollback" => rollback_changes(&args[2..]),
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    } else {
        // Launch GUI mode
        cerebras_maker_lib::run()
    }
}

fn print_help() {
    println!(
        r#"
Cerebras-MAKER: Autonomous Coding System
=========================================

USAGE:
    cerebras-maker [COMMAND] [OPTIONS]

COMMANDS:
    run <script.rhai>     Execute a Rhai script in headless mode
    analyze <path>        Analyze workspace topology and report red flags
    snapshot <message>    Create a git snapshot with message
    rollback [commit]     Rollback to previous snapshot or specific commit

OPTIONS:
    -h, --help            Print this help message
    -v, --version         Print version information

EXAMPLES:
    cerebras-maker                          # Launch GUI
    cerebras-maker run task.rhai            # Execute script
    cerebras-maker analyze ./src            # Analyze codebase
    cerebras-maker snapshot "Before refactor"
    cerebras-maker rollback
"#
    );
}

fn print_version() {
    println!("Cerebras-MAKER v{}", env!("CARGO_PKG_VERSION"));
}

fn run_script(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No script file provided");
        eprintln!("Usage: cerebras-maker run <script.rhai>");
        std::process::exit(1);
    }

    let script_path = PathBuf::from(&args[0]);
    if !script_path.exists() {
        eprintln!("Error: Script file not found: {}", script_path.display());
        std::process::exit(1);
    }

    println!("🚀 Executing script: {}", script_path.display());

    // Read and execute the script
    match std::fs::read_to_string(&script_path) {
        Ok(script) => {
            let workspace = env::current_dir().unwrap_or_default();
            let workspace_str = workspace.to_string_lossy();

            match cerebras_maker_lib::CodeModeRuntime::new(&workspace_str) {
                Ok(runtime) => {
                    match runtime.execute_script(&script) {
                        Ok(result) => {
                            println!("✅ Script completed successfully");
                            println!("Result: {:?}", result);
                        }
                        Err(e) => {
                            eprintln!("❌ Script execution failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize runtime: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading script: {}", e);
            std::process::exit(1);
        }
    }
}

fn analyze_workspace(args: &[String]) {
    let workspace = if args.is_empty() {
        env::current_dir().unwrap_or_default()
    } else {
        PathBuf::from(&args[0])
    };

    println!("🔍 Analyzing workspace: {}", workspace.display());

    // Use grits-core for analysis
    use grits_core::topology::scanner::DirectoryScanner;
    use grits_core::topology::analysis::TopologicalAnalysis;

    let scanner = DirectoryScanner::new();
    match scanner.scan(&workspace) {
        Ok(graph) => {
            let analysis = TopologicalAnalysis::analyze(&graph);

            println!("\n📊 Topology Analysis Results:");
            println!("   Symbols: {}", graph.nodes.len());
            println!("   Dependencies: {}", graph.edges.len());
            println!("   Betti-0 (Components): {}", analysis.betti_0);
            println!("   Betti-1 (Cycles): {}", analysis.betti_1);
            println!("   Triangles: {}", analysis.triangle_count);

            // Calculate solid score
            let solid = analysis.solid_score();
            println!("   Solid Score: {:.2}", solid.normalized);

            if analysis.betti_1 > 0 {
                println!("\n🚩 RED FLAG: {} dependency cycles detected!", analysis.betti_1);
                std::process::exit(1);
            } else {
                println!("\n✅ No architectural violations detected");
            }
        }
        Err(e) => {
            eprintln!("Error scanning workspace: {}", e);
            std::process::exit(1);
        }
    }
}

fn create_snapshot(args: &[String]) {
    let message = if args.is_empty() {
        "MAKER snapshot".to_string()
    } else {
        args.join(" ")
    };

    println!("📸 Creating snapshot: {}", message);

    let workspace = env::current_dir().unwrap_or_default();
    let workspace_str = workspace.to_string_lossy();
    let mut shadow = cerebras_maker_lib::ShadowGit::new(&workspace_str);

    if let Err(e) = shadow.init() {
        eprintln!("❌ Failed to initialize shadow git: {}", e);
        std::process::exit(1);
    }

    match shadow.snapshot(&message) {
        Ok(snapshot) => {
            println!("✅ Snapshot created: {}", snapshot.id);
        }
        Err(e) => {
            eprintln!("❌ Failed to create snapshot: {}", e);
            std::process::exit(1);
        }
    }
}

fn rollback_changes(args: &[String]) {
    let workspace = env::current_dir().unwrap_or_default();
    let workspace_str = workspace.to_string_lossy();
    let mut shadow = cerebras_maker_lib::ShadowGit::new(&workspace_str);

    if let Err(e) = shadow.init() {
        eprintln!("❌ Failed to initialize shadow git: {}", e);
        std::process::exit(1);
    }

    if args.is_empty() {
        println!("⏪ Rolling back to previous snapshot...");
        match shadow.rollback() {
            Ok(()) => println!("✅ Rollback successful"),
            Err(e) => {
                eprintln!("❌ Rollback failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let commit = &args[0];
        println!("⏪ Rolling back to commit: {}", commit);
        match shadow.rollback_to(commit) {
            Ok(()) => println!("✅ Rollback successful"),
            Err(e) => {
                eprintln!("❌ Rollback failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}
