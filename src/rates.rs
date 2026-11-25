use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ECB_DAILY_URL: &str = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml";

pub fn cache_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME")
        && !xdg.trim().is_empty()
    {
        return Some(PathBuf::from(xdg).join("omnicalc").join("rates.json"));
    }
    std::env::var("HOME")
        .ok()
        .filter(|h| !h.trim().is_empty())
        .map(|h| {
            PathBuf::from(h)
                .join(".cache")
                .join("omnicalc")
                .join("rates.json")
        })
}

fn extract_attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    for quote in ['\'', '"'] {
        let pat = format!("{}={}", key, quote);
        if let Some(start) = line.find(&pat) {
            let rest = &line[start + pat.len()..];
            if let Some(end) = rest.find(quote) {
                return Some(&rest[..end]);
            }
        }
    }
    None
}

/// Parses ECB `eurofxref-daily.xml` and normalizes all currency scales to 1 USD.
pub fn parse_ecb_xml(xml: &str) -> Result<(Option<String>, HashMap<String, f64>), String> {
    let mut updated: Option<String> = None;
    let mut eur_rates: HashMap<String, f64> = HashMap::new();

    for segment in xml.split('<') {
        if updated.is_none()
            && segment.contains("time=")
            && let Some(date) = extract_attr(segment, "time")
        {
            updated = Some(date.to_string());
        }
        if segment.contains("currency=")
            && segment.contains("rate=")
            && let (Some(curr), Some(rate_str)) = (
                extract_attr(segment, "currency"),
                extract_attr(segment, "rate"),
            )
            && let Ok(rate) = rate_str.parse::<f64>()
            && rate.is_finite()
            && rate > 0.0
        {
            eur_rates.insert(curr.to_string(), rate);
        }
    }

    let usd_per_eur = *eur_rates
        .get("USD")
        .ok_or_else(|| "ECB XML feed missing valid USD rate".to_string())?;

    let mut usd_scales = HashMap::new();
    usd_scales.insert("USD".to_string(), 1.0);
    usd_scales.insert("EUR".to_string(), usd_per_eur);
    for (curr, rate_per_eur) in eur_rates {
        if curr != "USD" && curr != "EUR" {
            usd_scales.insert(curr, usd_per_eur / rate_per_eur);
        }
    }

    Ok((updated, usd_scales))
}

pub fn write_rates_cache(
    path: &Path,
    updated: Option<&str>,
    rates: &HashMap<String, f64>,
) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Invalid cache file path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| {
        format!(
            "Failed to create cache directory '{}': {}",
            parent.display(),
            e
        )
    })?;

    let mut keys: Vec<&String> = rates.keys().collect();
    keys.sort();

    let mut json = String::from("{\n");
    json.push_str("  \"base\": \"USD\",\n");
    json.push_str("  \"source\": \"ECB\",\n");
    let date_val = updated.unwrap_or("unknown");
    json.push_str(&format!("  \"updated\": \"{}\",\n", date_val));
    json.push_str("  \"rates\": {\n");
    for (idx, k) in keys.iter().enumerate() {
        let comma = if idx + 1 < keys.len() { "," } else { "" };
        json.push_str(&format!("    \"{}\": {:.8}{}\n", k, rates[*k], comma));
    }
    json.push_str("  }\n}\n");

    let tmp_path = parent.join(format!(".rates.json.tmp.{}", std::process::id()));
    fs::write(&tmp_path, json)
        .map_err(|e| format!("Failed to write temporary rates cache: {}", e))?;
    fs::rename(&tmp_path, path)
        .map_err(|e| format!("Failed to finalize rates cache '{}': {}", path.display(), e))?;
    Ok(())
}

pub fn load_cached_rates_from(path: &Path) -> Option<HashMap<String, f64>> {
    let content = fs::read_to_string(path).ok()?;
    let rates_pos = content.find("\"rates\"")?;
    let brace_start = content[rates_pos..].find('{')? + rates_pos + 1;
    let brace_end = content[brace_start..].find('}')? + brace_start;
    let body = &content[brace_start..brace_end];

    let mut rates = HashMap::new();
    for entry in body.split(',') {
        let mut parts = entry.splitn(2, ':');
        let raw_key = parts.next()?.trim();
        let raw_val = parts.next()?.trim();
        let key = raw_key.trim_matches('"');
        if key.is_empty() {
            continue;
        }
        if let Ok(val) = raw_val.parse::<f64>()
            && val.is_finite()
            && val > 0.0
        {
            rates.insert(key.to_string(), val);
        }
    }
    if rates.is_empty() { None } else { Some(rates) }
}

pub fn load_cached_rates() -> Option<HashMap<String, f64>> {
    let path = cache_path()?;
    load_cached_rates_from(&path)
}

pub fn update_rates() -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "10", ECB_DAILY_URL])
        .output()
        .map_err(|e| format!("Failed to execute 'curl' for ECB rate update: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Failed to download ECB rates (exit {:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    let xml = String::from_utf8(output.stdout)
        .map_err(|e| format!("ECB feed returned invalid UTF-8: {}", e))?;
    let (updated, rates) = parse_ecb_xml(&xml)?;
    let path = cache_path()
        .ok_or_else(|| "Could not determine HOME or XDG_CACHE_HOME for rates cache".to_string())?;
    write_rates_cache(&path, updated.as_deref(), &rates)?;

    Ok(format!(
        "Updated {} currency rates from ECB (date: {}) -> {}",
        rates.len(),
        updated.as_deref().unwrap_or("unknown"),
        path.display()
    ))
}
