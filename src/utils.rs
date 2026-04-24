use std::io::{self, Read};
use comfy_table::Table;
use colored::*;
use crate::models::HeartRateZones;

pub fn read_stdin() -> Option<String> {
    if !atty::is(atty::Stream::Stdin) {
        let mut buffer = String::new();
        if io::stdin().read_to_string(&mut buffer).is_ok() {
            let s = buffer.trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.set_header(headers);
    for row in rows {
        table.add_row(row);
    }
    println!("{}", table);
}

pub fn format_duration(seconds: u32) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

pub fn format_pace(min_per_km: f64) -> String {
    let mins = min_per_km.floor() as u32;
    let secs = ((min_per_km - mins as f64) * 60.0).round() as u32;
    format!("{}'{:02}\"/km", mins, secs)
}

pub fn format_hr_zones_bar(zones: &HeartRateZones) -> String {
    let total_secs: u32 = zones.z1_seconds + zones.z2_seconds + zones.z3_seconds + zones.z4_seconds + zones.z5_seconds;
    if total_secs == 0 {
        return "No HR data".to_string();
    }

    let width: usize = 20;
    let z1_p = zones.z1_seconds as f64 / total_secs as f64;
    let z2_p = zones.z2_seconds as f64 / total_secs as f64;
    let z3_p = zones.z3_seconds as f64 / total_secs as f64;
    let z4_p = zones.z4_seconds as f64 / total_secs as f64;
    let z5_p = zones.z5_seconds as f64 / total_secs as f64;

    let z1_w = (z1_p * width as f64).round() as usize;
    let z2_w = (z2_p * width as f64).round() as usize;
    let z3_w = (z3_p * width as f64).round() as usize;
    let z4_w = (z4_p * width as f64).round() as usize;
    let z5_w = width.saturating_sub(z1_w + z2_w + z3_w + z4_w);

    let bar = format!("{}{}{}{}{}", 
        "█".repeat(z1_w).cyan(),
        "█".repeat(z2_w).green(),
        "█".repeat(z3_w).yellow(),
        "█".repeat(z4_w).magenta(),
        "█".repeat(z5_w).red()
    );

    format!("{} (Z1:{:.0}% Z2:{:.0}% Z3:{:.0}% Z4:{:.0}% Z5:{:.0}%)", 
        bar, z1_p * 100.0, z2_p * 100.0, z3_p * 100.0, z4_p * 100.0, z5_p * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table_smoke() {
        // Just verify it doesn't panic
        let headers = vec!["ID", "Name"];
        let rows = vec![vec!["1".to_string(), "Test".to_string()]];
        print_table(headers, rows);
    }
}
