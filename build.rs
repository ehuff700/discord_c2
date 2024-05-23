use std::io::Write;

fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN environment variable");
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let file = std::fs::File::create(format!("{out_dir}/constants.rs")).unwrap();
    writeln!(&file, "
    use once_cell::sync::Lazy;
    pub static DISCORD_TOKEN: Lazy<String> = Lazy::new(|| lc!(\"{}\"));
    ", token).unwrap();
    
}