// This example shows how to use pocketsphinx-rs to parse a JSGF-Grammar and can be run with `cargo run --example parse_jsgf`.

use pocketsphinx::{LogMath, JSGF};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let commands_jsgf_path = format!("{}/examples/data/commands.jsgf", manifest_dir);
    let jsgf = JSGF::from_file(commands_jsgf_path.as_str(), None)?;
    println!("JSGF name: {}", jsgf.get_name());
    let public_rule = jsgf.get_public_rule();
    match public_rule {
        Some(rule) => println!("Public rule: {}", rule.get_name()),
        None => println!("No public rule"),
    }
    let rules = jsgf.get_rule_iter();
    for rule in rules {
        println!("Rule: {}, Puplic: {}", rule.get_name(), rule.is_public());
    }
    // Let's test if the grammar matches some input
    let public_rule = jsgf.get_public_rule().unwrap();
    let logmath = LogMath::new(10.0, 0, false);
    let fsg = jsgf.build_fsg(&public_rule, &logmath, 1.0);
    println!(
        "Accepts 'turn on the lights': {}",
        fsg.accept("turn on the lights")
    );
    println!(
        "Accepts 'turn the lights off': {}",
        fsg.accept("turn the lights off")
    );
    println!("Accepts 'hello world': {}", fsg.accept("hello world"));
    Ok(())
}
