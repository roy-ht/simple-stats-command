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
                let (key, precision) = parse_placeholder(inner);

                if na_keys.contains(&key) {
                    result.push_str("N/A");
                } else if let Some(&val) = values.get(key) {
                    let p = precision.unwrap_or_else(|| default_precision(key));
                    result.push_str(&format!("{val:.prec$}", prec = p));
                } else if let Some(s) = str_values.get(key) {
                    result.push_str(s);
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

fn parse_placeholder(inner: &str) -> (&str, Option<usize>) {
    if let Some(colon_pos) = inner.find(":.") {
        let key = &inner[..colon_pos];
        let prec_str = &inner[colon_pos + 2..];
        let precision = prec_str.parse::<usize>().ok();
        (key, precision)
    } else {
        (inner, None)
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
    NeededMetrics {
        cpu: format.contains("{cpu"),
        memory: format.contains("{mem_") || format.contains("{swap_"),
        network: format.contains("{net_"),
        gpu: format.contains("{gpu_") || format.contains("{cuda_"),
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
        let result = render("{gpu_temp}C", &values, &str_values, &["gpu_temp"]);
        assert_eq!(result, "N/AC");
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
