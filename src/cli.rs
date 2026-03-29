const DEFAULT_FORMAT: &str = "{cpu}% | {mem_used:.1}/{mem_total:.1}G | {net_down:.1}/{net_up:.1}M/s";

pub struct Cli {
    pub format: String,
    pub interface: Option<String>,
    pub cache_path: Option<String>,
}

pub fn parse() -> Cli {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cli = Cli {
        format: DEFAULT_FORMAT.to_string(),
        interface: None,
        cache_path: None,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-f" | "--format" => {
                i += 1;
                cli.format = expect_value(&args, i, "--format");
            }
            "-I" | "--interface" => {
                i += 1;
                cli.interface = Some(expect_value(&args, i, "--interface"));
            }
            "--cache-path" => {
                i += 1;
                cli.cache_path = Some(expect_value(&args, i, "--cache-path"));
            }
            other => {
                eprintln!("unknown option: {other}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    cli
}

fn expect_value(args: &[String], i: usize, flag: &str) -> String {
    match args.get(i) {
        Some(v) => v.clone(),
        None => {
            eprintln!("{flag} requires a value");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    print!(
        "\
simple-stats - Lightweight system stats for terminal status lines

USAGE:
    simple-stats [OPTIONS]

OPTIONS:
    -f, --format <TEMPLATE>    Output format template [default: {default}]
    -I, --interface <NAME>     Network interface to monitor (default: all)
        --cache-path <PATH>    Cache file path (default: /tmp/simple-stats.bin)
    -h, --help                 Print help

PLACEHOLDERS:
    {{cpu}}          CPU usage (%)
    {{mem_used}}     Memory used (GiB)     {{mem_total}}    Memory total (GiB)
    {{mem_percent}}  Memory usage (%)      {{swap_used}}    Swap used (GiB)
    {{swap_total}}   Swap total (GiB)
    {{net_down}}     Download (MB/s)       {{net_up}}       Upload (MB/s)
    {{gpu_temp}}     GPU temp (C)          {{gpu_util}}     GPU usage (%)
    {{gpu_mem_used}} VRAM used (GiB)       {{gpu_mem_total}} VRAM total (GiB)
    {{gpu_name}}     GPU model name

    Precision: {{mem_used:.2}} for 2 decimal places
",
        default = DEFAULT_FORMAT
    );
}
