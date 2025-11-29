use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct TimeInput(u64);

impl TimeInput {
    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

impl FromStr for TimeInput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(val) = s.parse::<u64>() {
            return Ok(TimeInput(val));
        }
        let split_idx = s
            .find(|c: char| !c.is_numeric())
            .ok_or_else(|| format!("Invalid duration: '{}'. Expected number+unit (e.g. 1h)", s))?;
        let (val_str, unit_str) = s.split_at(split_idx);
        let value = val_str
            .parse::<u64>()
            .map_err(|_| format!("Invalid number in duration: '{}'", val_str))?;

        let multiplier: u64 = match unit_str.trim() {
            "ns" => return Ok(TimeInput(value / 1_000_000)),
            "us" | "µs" => return Ok(TimeInput(value / 1_000)),
            "ms" => 1,
            "s" | "sec" => 1_000,
            "m" | "min" => 60_000,
            "h" | "hr" => 3_600_000,
            "d" | "day" => 86_400_000,
            "w" | "wk" => 604_800_000,
            _ => return Err(format!("Unknown unit '{}'", unit_str)),
        };
        Ok(TimeInput(value * multiplier))
    }
}
