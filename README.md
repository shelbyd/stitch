# stitches

## Goal

stitches aims to empower everyone to solve mathematics problems with the most efficient use of the hardware they have available.

## Basic use

Create a new rust binary.

```
cargo new --bin <project_name>
```

Add the stitches library to Cargo.toml.

```
[dependencies]
stitches "*"
```

Copy the `null_problem` example to your main.rs file.

```sh
cp /path/to/stitches/examples/null_problem.rs src/main.rs
```

Run with `--release`.

```
cargo run --release
```

## Features

### Performance-focused

The goal of stitches can be more technically stated as putting your candidate checking in the hot path.
We only use monomorphization to maximize the percent of CPU time spent calculating your problem.

### Custom Spaces

If your problem requires searching a space we didn't think about, you can implement your own `Space` struct and use the same infrastructure as our out-of-the-box spaces.
We're even planning to include sanity and performance checks to let you know if your implementation may have problems.

## Roadmap

[x] Generic searching
[x] Basic performance statistics
[ ] Sanity/performance checks for Space implementation before starting search
[ ] Save/resume search state to/from disk
[ ] Multi-machine networked searching
[ ] Time to completion for finite spaces

### Spaces

[x] Linear
[ ] Tree
[ ] Multidimensional linear
