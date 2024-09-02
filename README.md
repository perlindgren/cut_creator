# Cut Creator

A tool to be, allowing the manipulation of turntable scratches.

## Requirements

The mold linker for improved build times under linux.

## Use

- Enable/disable cut lanes: click #0-#9 icon or press key 0-9.
- Load sample or cut: double click #0-#9 icon or press CTRL-O.
- Select knot: click to toggle select.
- Select knots: right mouse and drag to make selection.
  
- Export sample: Click Sample button.

## Development

This crate uses the `env_logger` crate for logging.

### Windows

In Powershell:

``` shell
$env:RUST_LOG="cut_creator=trace"
cargo run --release
```

## License

To be determined: Open source and (at least free for non-commercial use)
