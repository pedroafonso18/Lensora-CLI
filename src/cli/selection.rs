use std::collections::BTreeSet;
use std::io::{self, Write};

use colored::Colorize;

use super::diff::ChangedFile;

pub fn choose_files(files: &[ChangedFile]) -> anyhow::Result<Vec<ChangedFile>> {
    if files.is_empty() {
        return Ok(Vec::new());
    }

    loop {
        println!();
        println!("██╗     ███████╗███╗   ██╗███████╗ ██████╗ ██████╗  █████╗ ");
        println!("██║     ██╔════╝████╗  ██║██╔════╝██╔═══██╗██╔══██╗██╔══██╗");
        println!("██║     █████╗  ██╔██╗ ██║███████╗██║   ██║██████╔╝███████║");
        println!("██║     ██╔══╝  ██║╚██╗██║╚════██║██║   ██║██╔══██╗██╔══██║");
        println!("███████╗███████╗██║ ╚████║███████║╚██████╔╝██║  ██║██║  ██║");
        println!("╚══════╝╚══════╝╚═╝  ╚═══╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝");
        println!();
        println!("{}", "┌────────────────────────────────────┐".cyan());
        println!("{}", "│        Select files to review      │".cyan().bold());
        println!("{}", "├────────────────────────────────────┤".cyan());

        for (index, file) in files.iter().enumerate() {
            println!("{}", format!("│ {:>2}. {}", index + 1, file.path.display()).cyan());
        }

        println!("{}", "├─────────────────────────────────────┤".cyan());
        println!("{}", "│  a              = all files         │".cyan());
        println!("{}", "│  1,3,5 or 2-4   = specific files    │".cyan());
        println!("{}", "│  enter          = same as all files │".cyan());
        println!("{}", "└─────────────────────────────────────┘".cyan());

        print!("{}", "Choose: ".bold().yellow());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() || input.eq_ignore_ascii_case("a") {
            return Ok(files.to_vec());
        }

        let indexes = parse_selection(input, files.len())?;

        if indexes.is_empty() {
            println!("{}", "No valid files selected. Try again.".red());
            continue;
        }

        let selected = indexes
            .into_iter()
            .map(|index| files[index].clone())
            .collect();

        return Ok(selected);
    }
}

fn parse_selection(input: &str, file_count: usize) -> anyhow::Result<Vec<usize>> {
    let mut indexes = BTreeSet::new();

    for token in input.split(',').map(str::trim).filter(|token| !token.is_empty()) {
        if let Some((start, end)) = token.split_once('-') {
            let start = parse_index(start, file_count)?;
            let end = parse_index(end, file_count)?;

            let (start, end) = if start <= end { (start, end) } else { (end, start) };

            for index in start..=end {
                indexes.insert(index);
            }
            continue;
        }

        indexes.insert(parse_index(token, file_count)?);
    }

    Ok(indexes.into_iter().collect())
}

fn parse_index(value: &str, file_count: usize) -> anyhow::Result<usize> {
    let parsed = value.parse::<usize>()?;

    if parsed == 0 || parsed > file_count {
        anyhow::bail!("selection {parsed} is out of range");
    }

    Ok(parsed - 1)
}

#[cfg(test)]
mod tests {
    use super::parse_selection;

    #[test]
    fn parses_single_items_and_ranges() {
        let indexes = parse_selection("1,3-4", 5).unwrap();

        assert_eq!(indexes, vec![0, 2, 3]);
    }

    #[test]
    fn sorts_reversed_ranges() {
        let indexes = parse_selection("4-2", 5).unwrap();

        assert_eq!(indexes, vec![1, 2, 3]);
    }
}