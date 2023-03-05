# binroots
![Crates.io](https://img.shields.io/crates/v/binroots)
![Crates.io](https://img.shields.io/crates/d/binroots)
![Docs.rs](https://img.shields.io/badge/docs.rs-d2991d?&logo=docs.rs)
![Crates.io](https://img.shields.io/crates/l/binroots)
![Liberapay patrons](https://img.shields.io/liberapay/patrons/reebcw)

Binroots is a (cross-platform!) crate that provides a simple and efficient way to save Rust data structures to disk. It allows you to save each field of a struct or enum variant as a separate file, making it easy to store reactive data, allowing end-users and hackers to watch individual files for changes and automate command-line tools for your app.

## Project Status
Writing the initial commit of this crate took me about 7 hours. There are no unit tests yet, and it requires nightly Rust. If you care about your code, please do not use this in production (yet!). I can't guarantee that your files will remain safe. If you're interested in the development of binroots, check out the [planned features](#planned-features) and follow my [Twitter](https://twitter.com/carterisonline/) (no promises of whatever else you'll see on there)

## Usage

Add it to your Cargo.toml:

```toml
[dependencies]
binroots = "0.1.0"
```

## Setting up a struct

To save a struct, annotate it with `#[binroots_struct]`:

```rust
use binroots::binroots_struct;

#[binroots_struct]
struct Status {
    connections: usize,
    is_online: bool,
    activity: Activity,
}
```

`#[binroots_struct]` Automatically derives `Debug`, `Default` and `serde::Serialize`. It wraps each field in `BinrootsField`, which allows saving of individual fields without having to serialize the entire struct.

## Setting up an enum

In the struct above, we use an enum named `Activity`. Here's how it can be defined:
```rust
#[binroots_enum]
enum Activity {
    None, // <- Automatically chosen as the default value
    Playing(String),
}
```

`#[binroots_enum]` Also automatically derives `Debug`, `Default` and `serde::Serialize`. Wrapper types aren't needed.

In order to satisfy `Default`, it also picks the first variant named either `None`, `Nothing`, `Default`, or `Empty`. **If you wish to use a different default type**, you may annotate the enum with `#[binroots_enum(manual)]`, and mark a unit variant with `#[default]`.


## Saving data

In this example, we initialize `status` using `Status::default` (generated by `#[binroots_struct]`)

When saving a struct annotated with `#[binroots_struct]`, it will save to a subfolder named after the struct in `kebab-case`. In this example, on Unix, it saves to `/tmp/<crate name>/status`, and `%LOCALAPPDATA\<crate name>\cache\status` on Windows.

```rust
fn main() -> Result<(), SaveError> {
    let mut status = Status::default();

    *status.is_online = true;
    status.save()?; // <- Saves the entire struct to the disk

    *status.activity = Activity::Playing("video gamb".into());
    status.activity.save(Status::ROOT_FOLDER)?; // <- Only saves status.activity to the disk

    Ok(())
}
```

After saving, the `status` folder should look like this:

```bash

/tmp/binroots/status
├── activity           => "Playing"
├── activity.value     => "video gamb"
├── connections        => "0"
└── is_online          => "true"
```

<h2 id="planned-features"> Planned Features </h2>

I'm most likely going to add these in order. If it's not on this list, it's either implemented or unconsidered.
- `#[binroots_struct(persistent)]` for saving to the disk (`$HOME/.cache/<crate name>` on Unix)
- Setting `hFile = INVALID_HANDLE_VALUE` on Windows when using in-memory storage. Currently can only save to persistent storage on Windows.
- Unit tests lol
- Union support
- Unify `#[binroots_*]` macros into a single `#[binroots]` macro
- `Self::enable_autosave(self) -> Self` for reactive data
- `Deserialize`
- `Send + Sync`
- `BinrootsField::watch(Fn(T))` for socket-ish behavior
- `Self::enable_rx(self) -> Self` for true two-way reactive data
- Async support?
- Dual free/commercial license