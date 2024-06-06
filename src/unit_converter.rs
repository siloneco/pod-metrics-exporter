pub fn parse_cpu_value(str: &str) -> Result<f64, String> {
    // if str is a number, return it
    if let Ok(parsed) = str.parse::<f64>() {
        return Ok(parsed);
    }

    // Get the suffix of the string and convert to the milli core
    let suffix = str.chars().last().unwrap().to_ascii_lowercase();
    let num_value = str[..str.len() - 1].parse::<f64>().unwrap();

    let result = match suffix {
        'n' => num_value / 1_000_000.0,
        'u' => num_value / 1_000.0,
        'm' => num_value,
        _ => {
            if num_value == 0.0 {
                0.0
            } else {
                return Err(format!("Unknown CPU unit: {}", str));
            }
        }
    };

    Ok(result)
}

pub fn parse_memory_value(str: &str) -> Result<f64, String> {
    // if str is a number, return it
    if let Ok(parsed) = str.parse::<f64>() {
        return Ok(parsed);
    }

    // Get the suffix of the string and convert to the MiB
    let suffix = &str[str.len() - 2..];
    let num_value = str[..str.len() - 2].parse::<f64>().unwrap();

    let result = match suffix {
        "Ki" => num_value / 1024.0,
        "Mi" => num_value,
        "Gi" => num_value * 1024.0,
        _ => {
            return Err(format!("Unknown memory unit: {}", str));
        }
    };

    Ok(result)
}
