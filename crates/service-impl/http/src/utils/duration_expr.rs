pub fn serialize<S>(duration: &tokio::time::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = format!("{}", DurationExprDisplay(duration));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<tokio::time::Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    from_duration_expr(s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub struct Never(());

impl serde::Serialize for Never {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("never")
    }
}

impl<'de> serde::Deserialize<'de> for Never {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        if s.eq_ignore_ascii_case("never") {
            Ok(Never(()))
        } else {
            Err(serde::de::Error::custom(format!(
                "expected 'never', found '{}'",
                s
            )))
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, bincode::Encode, bincode::Decode)]
#[serde(untagged)]
/// Represents a timeout duration which can be specified as an expression, milliseconds, or never.
/// - `Expr`: A human-readable duration expression (e.g., "1h 30m").
/// - `MilliSecond`: A duration specified in milliseconds, represented as a u32.
/// - `Never`: Represents an infinite timeout, should be serialized/deserialized as the string "never".
pub enum TimeoutDuration {
    Never(Never),
    Expr(#[serde(with = "self")] tokio::time::Duration),
    MilliSecond(u32),
}

impl Default for TimeoutDuration {
    fn default() -> Self {
        TimeoutDuration::Expr(tokio::time::Duration::from_secs(30))
    }
}

impl TimeoutDuration {
    pub fn as_duration(&self) -> Option<std::time::Duration> {
        match self {
            TimeoutDuration::Never(_) => None,
            TimeoutDuration::Expr(dur) => Some(
                std::time::Duration::from_secs(dur.as_secs())
                    .checked_add(std::time::Duration::from_nanos(dur.subsec_nanos() as u64))
                    .unwrap(),
            ),
            TimeoutDuration::MilliSecond(ms) => Some(std::time::Duration::from_millis(*ms as u64)),
        }
    }
}

pub struct DurationExprDisplay<'d>(pub &'d tokio::time::Duration);

impl<'d> std::fmt::Display for DurationExprDisplay<'d> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt_duration_expr(self.0, f)
    }
}

pub fn fmt_duration_expr(
    duration: &tokio::time::Duration,
    f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    // convert to H:M:S format
    let total_secs = duration.as_secs();
    let nano_secs = duration.subsec_nanos();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = nano_secs / 1_000_000;

    if hours > 0 {
        write!(f, "{}h {:02}m {:02}s", hours, minutes, seconds)?;
    } else if minutes > 0 {
        write!(f, "{}m {:02}s", minutes, seconds)?;
    } else if seconds > 0 {
        write!(f, "{}s", seconds)?;
        if millis > 0 {
            write!(f, " {}ms", millis)?;
        }
    } else {
        write!(f, "{}ms", millis)?;
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DurationExprParseError {
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
    #[error("Invalid unit: {0}")]
    InvalidUnit(String),
}
pub fn from_duration_expr(
    duration_str: &str,
) -> Result<tokio::time::Duration, DurationExprParseError> {
    let duration_str = duration_str.trim();
    let mut total_millis: u64 = 0;
    let mut num_buf = String::new();
    let mut unit_buf = String::new();

    for c in duration_str.chars().chain(std::iter::once(' ')) {
        if c.is_digit(10) {
            if !unit_buf.is_empty() {
                // Process previous number and unit
                let num: u64 = num_buf
                    .parse()
                    .map_err(|_| DurationExprParseError::InvalidNumber(num_buf.clone()))?;
                match unit_buf.as_str() {
                    "h" => total_millis += num * 3600 * 1000,
                    "m" => total_millis += num * 60 * 1000,
                    "s" => total_millis += num * 1000,
                    "ms" => total_millis += num,
                    _ => return Err(DurationExprParseError::InvalidUnit(unit_buf.clone())),
                }
                num_buf.clear();
                unit_buf.clear();
            }
            num_buf.push(c);
        } else if !c.is_whitespace() {
            unit_buf.push(c);
        }
    }

    if !num_buf.is_empty() && !unit_buf.is_empty() {
        let num: u64 = num_buf
            .parse()
            .map_err(|_| DurationExprParseError::InvalidNumber(num_buf.clone()))?;
        match unit_buf.as_str() {
            "h" => total_millis += num * 3600 * 1000,
            "m" => total_millis += num * 60 * 1000,
            "s" => total_millis += num * 1000,
            "ms" => total_millis += num,
            _ => return Err(DurationExprParseError::InvalidUnit(unit_buf.clone())),
        }
    }

    Ok(tokio::time::Duration::from_millis(total_millis))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_duration_expr() {
        let tests = vec![
            ("1h 30m", 5400_000),
            ("1h30m", 5400_000),
            ("2m 15s", 135_000),
            ("2m15s", 135_000),
            ("45s", 45_000),
            ("500ms", 500),
            ("1h 2m 3s 400ms", 3723_400),
            ("3s 250ms", 3_250),
            ("0s 750ms", 750),
        ];

        for (input, expected_millis) in tests {
            let duration = from_duration_expr(input).unwrap();
            assert_eq!(duration.as_millis() as u64, expected_millis);

            let output = format!("{}", DurationExprDisplay(&duration));
            let parsed_duration = from_duration_expr(&output).unwrap();
            assert_eq!(parsed_duration, duration);
        }
    }

    #[test]
    fn test_timeout_duration_deserialize() {
        let json_expr = r#""1h 15m""#;
        let timeout: TimeoutDuration = serde_json::from_str(json_expr).unwrap();
        match timeout {
            TimeoutDuration::Expr(dur) => {
                assert_eq!(dur.as_secs(), 4500);
            }
            _ => panic!("Expected Expr variant"),
        }
        let json_expr = r#""300ms""#;
        let timeout: TimeoutDuration = serde_json::from_str(json_expr).unwrap();
        match timeout {
            TimeoutDuration::Expr(dur) => {
                assert_eq!(dur.as_millis(), 300);
            }
            _ => panic!("Expected Expr variant"),
        }
        let json_millis = r#"900000"#;
        let timeout: TimeoutDuration = serde_json::from_str(json_millis).unwrap();
        match timeout {
            TimeoutDuration::MilliSecond(ms) => {
                assert_eq!(ms, 900000);
            }
            _ => panic!("Expected MilliSecond variant"),
        }

        let json_never = r#""Never""#;
        let timeout: TimeoutDuration = serde_json::from_str(json_never).unwrap();
        match timeout {
            TimeoutDuration::Never(_) => {}
            _ => panic!("Expected Never variant, but got {:?}", timeout),
        }
    }
}
