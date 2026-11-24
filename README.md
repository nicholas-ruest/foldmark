# foldmark

`foldmark` is a small Rust CLI and library for arranging document pages into
saddle-stitched booklet signatures. It tells you exactly which pages belong on
the left and right of every sheet side, splits thick booklets into manageable
signatures, and makes padding pages visible instead of silently losing them.

It does not modify a PDF. Use the table, CSV, or JSON plan to configure manual
duplex printing, a print-shop workflow, or another document tool.

## Install

```bash
cargo install --git https://github.com/nicholas-ruest/foldmark --locked
```

Or clone the repository and run it directly:

```bash
cargo run --release -- 28 --signature-sheets 4
```

## Use

Plan a 28-page, left-bound booklet with at most four sheets (16 pages) in each
signature:

```bash
foldmark 28 --signature-sheets 4
```

```text
28 pages -> 7 sheets, 2 signatures, 0 blank positions
sig  sheet  side   left       right
---  -----  -----  ---------  ---------
1    1      front  16         1
1    1      back   2          15
...
2    3      back   22         23
```

Export a machine-readable plan, mirror every spread for a right-bound book,
or save CSV for a spreadsheet:

```bash
foldmark 19 --format json
foldmark 12 --binding right
foldmark 40 --signature-sheets 3 --format csv --output plan.csv
```

The first physical sheet listed for each signature is the outermost sheet.
`front` and `back` describe the two printed sides; `left` and `right` are the
positions as that side faces you. Confirm your printer's duplex flip setting
with a one-sheet test because feed orientation varies by printer.

## What works

- correct outside-to-inside imposition for saddle stitching;
- bounded multi-signature splitting;
- explicit blank positions for page counts not divisible by four;
- left- and right-bound layouts;
- aligned terminal tables plus stable CSV and JSON;
- reusable safe-Rust library API with overflow-checked arithmetic.

## Limits

`foldmark` plans page order only. It does not rasterize, resize, rotate, or
write PDF files, compensate for paper creep, or control a printer. Signatures
are filled to the requested maximum before the final shorter signature.

## Development

The project targets stable Rust and declares Rust 1.85 as its minimum supported
version.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## License

MIT
