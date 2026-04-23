use std::io::{self, Read};
use comfy_table::Table;

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
