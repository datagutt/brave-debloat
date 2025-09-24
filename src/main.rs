use clap::Parser;

use brave_debloater::{
    Args, DebloaterError, DebloaterGenerator,
    load_config, load_extensions, load_preferences_config,
    Platform
};

fn main() -> Result<(), DebloaterError> {
    let args = Args::parse();
    
    println!("Loading configuration from: {}", args.config);
    let config = load_config(&args.config)?;
    
    println!("Loading extensions from: {}", args.extensions);
    let extensions = load_extensions(&args.extensions)?;
    
    let extension_names: Vec<String> = extensions.iter().map(|e| e.name.clone()).collect();
    println!("Loaded {} extensions: {}", 
             extensions.len(),
             extension_names.join(", "));

    // Always load preferences config for unified scripts
    println!("Loading preferences from: {}", args.preferences_config);
    let preferences_config = load_preferences_config(&args.preferences_config)?;

    if preferences_config.is_some() {
        println!("Loaded preferences configuration");
    } else {
        println!("Using default preferences configuration");
    }
    
    println!("Generating unified {} script for Brave {:?}...", 
             match args.platform {
                 Platform::Windows => "Windows",
                 Platform::MacOS => "macOS",
                 Platform::Linux => "Linux",
             },
             args.version);
    
    let output_dir = args.output.clone();
    let generator = DebloaterGenerator::new(config, extensions, args.platform, args.version, args.output, preferences_config);
    generator.generate()?;
    
    println!("Configuration files generated successfully in: {}", output_dir);
    
    Ok(())
}