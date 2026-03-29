use std::collections::HashMap;

pub fn render(
    template: &str,
    values: &HashMap<String, f64>,
    str_values: &HashMap<String, String>,
    na_keys: &[&str],
) -> String {
    let mut result = String::with_capacity(template.len() * 2);
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'{' {
            if let Some(close) = template[i..].find('}') {
                let inner = &template[i + 1..i + close];
                let ph = parse_placeholder(inner);

                if na_keys.contains(&ph.key) {
                    // N/A: render nothing (prefix and suffix are also suppressed)
                } else if let Some(&val) = values.get(ph.key) {
                    let p = ph.precision.unwrap_or_else(|| default_precision(ph.key));
                    result.push_str(ph.prefix);
                    result.push_str(&format!("{val:.prec$}", prec = p));
                    result.push_str(ph.suffix);
                } else if let Some(s) = str_values.get(ph.key) {
                    result.push_str(ph.prefix);
                    result.push_str(s);
                    result.push_str(ph.suffix);
                } else {
                    result.push('{');
                    result.push_str(inner);
                    result.push('}');
                }
                i += close + 1;
            } else {
                result.push(bytes[i] as char);
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

struct Placeholder<'a> {
    prefix: &'a str,
    key: &'a str,
    precision: Option<usize>,
    suffix: &'a str,
}

/// Parse a placeholder of the form: ["prefix"]key[:.N]["suffix"]
///
/// Quoted string literals (`"..."`) immediately before or after the key:precision
/// are treated as prefix/suffix that disappear when the value is N/A.
///
/// Examples:
///   `cpu:.1`              → key="cpu", precision=Some(1), prefix="", suffix=""
///   `"  "gpu_name`        → key="gpu_name", precision=None, prefix="  ", suffix=""
///   `" "gpu_util:.1" %"`  → key="gpu_util", precision=Some(1), prefix=" ", suffix=" %"
fn parse_placeholder(inner: &str) -> Placeholder<'_> {
    let mut rest = inner;

    // Optional prefix: "..."
    let prefix = if rest.starts_with('"') {
        if let Some(end) = rest[1..].find('"') {
            let p = &rest[1..end + 1];
            rest = &rest[end + 2..];
            p
        } else {
            ""
        }
    } else {
        ""
    };

    // Key: letters, digits, underscore
    let key_end = rest
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    let key = &rest[..key_end];
    rest = &rest[key_end..];

    // Optional precision: :.N
    let precision = if rest.starts_with(":.") {
        let prec_str = &rest[2..];
        let prec_end = prec_str
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(prec_str.len());
        let p = prec_str[..prec_end].parse::<usize>().ok();
        rest = &prec_str[prec_end..];
        p
    } else {
        None
    };

    // Optional suffix: "..."
    let suffix = if rest.starts_with('"') {
        if let Some(end) = rest[1..].find('"') {
            &rest[1..end + 1]
        } else {
            ""
        }
    } else {
        ""
    };

    Placeholder {
        prefix,
        key,
        precision,
        suffix,
    }
}

fn default_precision(key: &str) -> usize {
    match key {
        "cpu" | "mem_percent" | "gpu_temp" | "gpu_util" => 0,
        _ => 1,
    }
}

/// Returns the set of metric prefixes needed based on the format string.
pub struct NeededMetrics {
    pub cpu: bool,
    pub memory: bool,
    pub network: bool,
    pub gpu: bool,
}

pub fn parse_needed(format: &str) -> NeededMetrics {
    // Extract all placeholder keys from the template (handles prefix/suffix syntax)
    let mut cpu = false;
    let mut memory = false;
    let mut network = false;
    let mut gpu = false;

    let bytes = format.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'{' {
            if let Some(close) = format[i..].find('}') {
                let inner = &format[i + 1..i + close];
                let ph = parse_placeholder(inner);
                let key = ph.key;
                if key == "cpu" {
                    cpu = true;
                } else if key.starts_with("mem_") || key.starts_with("swap_") {
                    memory = true;
                } else if key.starts_with("net_") {
                    network = true;
                } else if key.starts_with("gpu_") || key.starts_with("cuda_") {
                    gpu = true;
                }
                i += close + 1;
                continue;
            }
        }
        i += 1;
    }

    NeededMetrics {
        cpu,
        memory,
        network,
        gpu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic() {
        let mut values = HashMap::new();
        values.insert("cpu".to_string(), 42.0);
        values.insert("mem_used".to_string(), 12.345);

        let str_values = HashMap::new();
        let result = render("{cpu}% | {mem_used:.1}G", &values, &str_values, &[]);
        assert_eq!(result, "42% | 12.3G");
    }

    #[test]
    fn test_render_na() {
        let values = HashMap::new();
        let str_values = HashMap::new();
        // N/A renders as empty string; text outside {} is unaffected
        let result = render("{gpu_temp}C", &values, &str_values, &["gpu_temp"]);
        assert_eq!(result, "C");
    }

    #[test]
    fn test_render_na_with_prefix_suffix() {
        let values = HashMap::new();
        let str_values = HashMap::new();
        // Prefix and suffix inside {} are also suppressed when N/A
        let result = render(r#"{"  "gpu_util:.1"%"}"#, &values, &str_values, &["gpu_util"]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_prefix_suffix() {
        let mut values = HashMap::new();
        values.insert("gpu_util".to_string(), 50.0);
        let str_values = HashMap::new();
        let result = render(r#"{"  "gpu_util:.1"%"}"#, &values, &str_values, &[]);
        assert_eq!(result, "  50.0%");
    }

    #[test]
    fn test_render_prefix_only() {
        let mut str_values = HashMap::new();
        str_values.insert("gpu_name".to_string(), "RTX 4090".to_string());
        let values = HashMap::new();
        let result = render(r#"{"  "gpu_name}"#, &values, &str_values, &[]);
        assert_eq!(result, "  RTX 4090");
    }

    #[test]
    fn test_render_string_value() {
        let values = HashMap::new();
        let mut str_values = HashMap::new();
        str_values.insert("gpu_name".to_string(), "RTX 4090".to_string());
        let result = render("{gpu_name}", &values, &str_values, &[]);
        assert_eq!(result, "RTX 4090");
    }

    #[test]
    fn test_render_unknown_placeholder() {
        let values = HashMap::new();
        let str_values = HashMap::new();
        let result = render("{unknown}", &values, &str_values, &[]);
        assert_eq!(result, "{unknown}");
    }
}
