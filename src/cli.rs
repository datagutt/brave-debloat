use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BraveVersion {
    Normal,
    Nightly,
}

#[derive(Parser, Debug)]
#[command(name = "brave-debloater")]
#[command(about = "A tool to generate Brave browser debloat configurations for different platforms")]
pub struct Args {
    #[arg(short, long, value_enum)]
    pub platform: Platform,
    
    #[arg(short, long, value_enum, default_value = "normal")]
    pub version: BraveVersion,
    
    #[arg(short, long, default_value = "configs/privacy-focused.json")]
    pub config: String,
    
    #[arg(short, long, default_value = "extensions.json")]
    pub extensions: String,
    
    #[arg(short, long, default_value = "output")]
    pub output: String,
    
    #[arg(long, default_value = "preferences.json", help = "Preferences configuration file")]
    pub preferences_config: String,
}