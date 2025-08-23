use raw_cpuid::CpuId;
use std::env;
use std::fs;
use std::time::Duration;

static RESET_CODE: &str = "\x1b[0m";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn help_menu() {
    println!(
        "rfetch {VERSION}

USAGE:
    rfetch [OPTIONS]

OPTIONS (optional):
    --config <FILE>     path to text file containing ascii art
    --spacing <N>       spaces before ASCII art (0–255, default=3)
    --color <ANSI>      (e.g. 36, 1;36, 38;5;205)
    -h, --help          print help
    -v, --version       print version"
    );
}

fn make_separator(length: usize, color_code: &str) -> String {
    format!("{}{}{}", color_code, "-".repeat(length), RESET_CODE)
}

// environment variable or default
fn evod(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn read_file_trim(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn read_os_release(path: &str) -> (String, String) {
    let content = read_file_trim(path);
    let mut os_name = "Unknown OS".to_string();
    let mut ansi_color = "0;37".to_string();

    for line in content.lines() {
        if line.starts_with("PRETTY_NAME=") {
            os_name = line
                .trim_start_matches("PRETTY_NAME=")
                .trim_matches('"')
                .to_string();
        }
        if line.starts_with("ANSI_COLOR=") {
            ansi_color = line
                .trim_start_matches("ANSI_COLOR=")
                .trim_matches('"')
                .to_string();
        }
    }

    (os_name, ansi_color)
}

fn color_ascii_art(ascii_art: &str, color_code: &str, spacing: u8) -> Vec<String> {
    let prefix = " ".repeat(spacing as usize);
    ascii_art
        .lines()
        .map(|line| format!("{}{}{}{}", prefix, color_code, line, RESET_CODE))
        .collect()
}

fn get_user() -> String {
    let username = evod("USER", "unknown");
    let hostname = read_file_trim("/proc/sys/kernel/hostname")
        .split('.')
        .next()
        .unwrap_or("unknown")
        .to_string();
    format!("{}@{}", username, hostname)
}

fn get_cpu() -> String {
    CpuId::new()
        .get_processor_brand_string()
        .map(|b| b.as_str().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string())
}

fn get_kernel() -> String {
    let kernel_type = read_file_trim("/proc/sys/kernel/ostype");
    let kernel_release = read_file_trim("/proc/sys/kernel/osrelease");
    format!("KERNEL: {} {}", kernel_type, kernel_release)
}

fn get_uptime() -> String {
    let uptime_seconds: f64 = read_file_trim("/proc/uptime")
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let duration = Duration::from_secs_f64(uptime_seconds);
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    format!("Uptime: {} hours, {} mins", hours, minutes)
}

fn print_stuff(ascii_lines: &[String], sys_info: &[String], spacing: u8) {
    let max_ascii_len = ascii_lines
        .iter()
        .map(|line| line.len() - spacing as usize)
        .max()
        .unwrap_or(0);
    let offset = max_ascii_len + 5 + spacing as usize;

    let max_lines = ascii_lines.len().max(sys_info.len());
    let empty: &str = "";
    for i in 0..max_lines {
        let art_line: &str = if i < ascii_lines.len() {
            &ascii_lines[i]
        } else {
            empty
        };
        let info_line: &str = if i < sys_info.len() { &sys_info[i] } else { "" };
        let spaces = " ".repeat(offset.saturating_sub(art_line.len()));
        println!("{}{}{}", art_line, spaces, info_line);
    }
}

// ------------------------- Main -------------------------

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        help_menu();
        return;
    } else if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("rfetch {}", VERSION);
        return;
    }

    let mut ascii_art = r#"        #####
       #######
       ##O#O##
       #######
     ###########
    #############
   ###############
   ################
  #################
#####################
#####################
  #################
"#
    .to_string();
    let mut spacing: u8 = 3;

    let (os_name, mut ansi_color) = if std::path::Path::new("/etc/os-release").exists() {
        read_os_release("/etc/os-release")
    } else if std::path::Path::new("/usr/lib/os-release").exists() {
        read_os_release("/usr/lib/os-release")
    } else {
        ("Unknown OS".to_string(), "0;37".to_string())
    };

    // parse args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                if i + 1 < args.len() {
                    ascii_art = read_file_trim(&args[i + 1]);
                    i += 1;
                }
            }
            "--spacing" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<u8>() {
                        spacing = val;
                    }
                    i += 1;
                }
            }
            "--color" => {
                if i + 1 < args.len() {
                    ansi_color = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let color_code = format!("\x1b[{}m", ansi_color);
    let colored_art_lines = color_ascii_art(&ascii_art, &color_code, spacing);

    let separator = make_separator(get_user().len(), &color_code);

    let sys_info = vec![
        get_user(),
        get_uptime(),
        separator.clone(),
        format!("OS: {}", os_name),
        format!("CPU: {}", get_cpu()),
        get_kernel(),
        separator.clone(),
        format!("Terminal: {}", evod("TERM", "unknown")),
        format!("Shell: {}", evod("SHELL", "unknown")),
        format!("WM: {}", evod("XDG_CURRENT_DESKTOP", "unknown")),
        separator.clone(),
    ];

    print_stuff(&colored_art_lines, &sys_info, spacing);
}
