use std::path::PathBuf;

use binroots::save::{RootType, Save};
use binroots::Serialize;

#[derive(Serialize)]
enum Either {
    One,
    Two(String),
}

#[derive(Serialize)]
struct UhOh {
    this: String,
    might: f32,
    not: bool,
    work: (),
    // it does!
}

#[derive(Serialize)]
struct Hello {
    world: String,
    num: u8,
    v: Vec<Either>,
    tuple: (u8, u16, f64),
    uhoh: UhOh,
}

fn main() {
    println!("{:?}", binroots::save::root_location(RootType::InMemory));
    let h = Hello {
        world: "world".into(),
        num: 1,
        v: vec![Either::One, Either::Two("Hi".into()), Either::One],
        tuple: (100, 5140, 3.14159),
        uhoh: UhOh {
            this: "this...".into(),
            might: 6.33,
            not: true,
            work: (),
        },
    };

    h.save(PathBuf::from("hello"), RootType::InMemory).unwrap();
}
