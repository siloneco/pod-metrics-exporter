pub fn parse_cpu_value(str: &str) -> Result<f64, String> {
    // check str length
    if str.len() == 0 {
        return Err(format!("Empty string cannot be parsed as CPU value"));
    }

    // if str is a number, return it
    if let Ok(parsed) = str.parse::<f64>() {
        return Ok(parsed);
    }

    // Get the suffix of the string and convert to the milli core
    let suffix = str.chars().last().unwrap().to_ascii_lowercase();
    let num_value = str[..str.len() - 1].parse::<f64>();

    let num_value = match num_value {
        Ok(value) => value,
        Err(_) => {
            return Err(format!("Unable to parse CPU value: {}", str));
        }
    };

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
    // check str length
    if str.len() == 0 {
        return Err(format!("Empty string cannot be parsed as memory value"));
    }

    // if str is a number, return it
    if let Ok(parsed) = str.parse::<f64>() {
        return Ok(parsed);
    }

    // str must be at least 3 characters long
    if str.len() < 3 {
        return Err(format!("Unable to parse memory value: {}", str));
    }

    // Get the suffix of the string and convert to the MiB
    let suffix = &str[str.len() - 2..];
    let num_value = str[..str.len() - 2].parse::<f64>();

    let num_value = match num_value {
        Ok(value) => value,
        Err(_) => {
            return Err(format!("Unable to parse memory value: {}", str));
        }
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_value() {
        assert_eq!(parse_cpu_value("0"), Ok(0.0));
        assert_eq!(parse_cpu_value("1"), Ok(1.0));
        assert_eq!(parse_cpu_value("1m"), Ok(1.0));
        assert_eq!(parse_cpu_value("1u"), Ok(0.001));
        assert_eq!(parse_cpu_value("1n"), Ok(0.000001));
        assert_eq!(
            parse_cpu_value("1xxx"),
            Err("Unable to parse CPU value: 1xxx".to_string())
        );
        assert_eq!(
            parse_cpu_value("1x"),
            Err("Unknown CPU unit: 1x".to_string())
        );
    }

    #[test]
    fn test_parse_memory_value() {
        assert_eq!(parse_memory_value("0"), Ok(0.0));
        assert_eq!(parse_memory_value("1"), Ok(1.0));
        assert_eq!(parse_memory_value("1Mi"), Ok(1.0));
        assert_eq!(parse_memory_value("1Ki"), Ok(1.0 / 1024.0));
        assert_eq!(parse_memory_value("1Gi"), Ok(1024.0));
        assert_eq!(
            parse_memory_value("1xxx"),
            Err("Unable to parse memory value: 1xxx".to_string())
        );
        assert_eq!(
            parse_memory_value("1x"),
            Err("Unable to parse memory value: 1x".to_string())
        );
        assert_eq!(
            parse_memory_value("123XX"),
            Err("Unknown memory unit: 123XX".to_string())
        );
    }
}
