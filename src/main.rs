use clap::{Parser, ValueEnum};
use foldmark::{plan_booklet, Binding, BookletPlan};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Plan page order for saddle-stitched booklet signatures",
    after_help = "Example: foldmark 28 --signature-sheets 4 --format csv --output plan.csv"
)]
struct Cli {
    /// Number of logical pages in the source document.
    #[arg(value_parser = clap::value_parser!(u32).range(1..))]
    pages: u32,

    /// Maximum physical sheets nested in each signature.
    #[arg(long, default_value_t = 4, value_parser = clap::value_parser!(u32).range(1..))]
    signature_sheets: u32,

    /// Spine side of the finished booklet.
    #[arg(long, value_enum, default_value_t = BindingArg::Left)]
    binding: BindingArg,

    /// Output representation.
    #[arg(long, value_enum, default_value_t = Format::Table)]
    format: Format,

    /// Write to a file instead of standard output.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BindingArg {
    Left,
    Right,
}

impl From<BindingArg> for Binding {
    fn from(value: BindingArg) -> Self {
        match value {
            BindingArg::Left => Self::Left,
            BindingArg::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Table,
    Csv,
    Json,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("foldmark: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let plan = plan_booklet(cli.pages, cli.signature_sheets, cli.binding.into())?;

    let writer: Box<dyn Write> = match cli.output {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(BufWriter::new(io::stdout().lock())),
    };

    match cli.format {
        Format::Table => write_table(&plan, writer)?,
        Format::Csv => write_csv(&plan, writer)?,
        Format::Json => serde_json::to_writer_pretty(writer, &plan)?,
    }
    Ok(())
}

fn write_table(plan: &BookletPlan, mut writer: impl Write) -> io::Result<()> {
    writeln!(
        writer,
        "{} pages -> {} sheets, {} signatures, {} blank positions",
        plan.document_pages,
        plan.total_sheets,
        plan.signatures.len(),
        plan.total_blank_positions
    )?;
    writeln!(
        writer,
        "sig  sheet  side   left       right\n---  -----  -----  ---------  ---------"
    )?;
    for signature in &plan.signatures {
        for sheet in &signature.sheets {
            writeln!(
                writer,
                "{:<4} {:<6} {:<6} {:<10} {}",
                signature.number, sheet.number, "front", sheet.front.left, sheet.front.right
            )?;
            writeln!(
                writer,
                "{:<4} {:<6} {:<6} {:<10} {}",
                signature.number, sheet.number, "back", sheet.back.left, sheet.back.right
            )?;
        }
    }
    Ok(())
}

fn write_csv(plan: &BookletPlan, mut writer: impl Write) -> io::Result<()> {
    writeln!(writer, "signature,sheet,side,left,right")?;
    for signature in &plan.signatures {
        for sheet in &signature.sheets {
            writeln!(
                writer,
                "{},{},front,{},{}",
                signature.number, sheet.number, sheet.front.left, sheet.front.right
            )?;
            writeln!(
                writer,
                "{},{},back,{},{}",
                signature.number, sheet.number, sheet.back.left, sheet.back.right
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use foldmark::Binding;

    #[test]
    fn table_contains_summary_and_page_order() {
        let plan = plan_booklet(4, 4, Binding::Left).unwrap();
        let mut output = Vec::new();
        write_table(&plan, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("4 pages -> 1 sheets, 1 signatures, 0 blank positions"));
        assert!(text.contains("front  4          1"));
    }

    #[test]
    fn csv_has_stable_header_and_rows() {
        let plan = plan_booklet(4, 4, Binding::Left).unwrap();
        let mut output = Vec::new();
        write_csv(&plan, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "signature,sheet,side,left,right\n1,1,front,4,1\n1,1,back,2,3\n"
        );
    }
}
