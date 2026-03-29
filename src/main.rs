mod cache;
mod cli;
mod format;
mod metrics;

fn main() {
    let args = cli::parse();
    let needed = format::parse_needed(&args.format);

    let cache_path = args
        .cache_path
        .unwrap_or_else(cache::default_cache_path);

    let result = metrics::collect_all(&metrics::CollectOptions {
        needed: &needed,
        interface: args.interface.as_deref(),
        cache_path: &cache_path,
    });

    let na_refs: Vec<&str> = result.na_keys.iter().map(|s| s.as_str()).collect();
    let output = format::render(&args.format, &result.values, &result.str_values, &na_refs);

    print!("{output}");
}
