use std::{env, error::Error, fs};

use alkahest::Formula;
use alkahest_schema::Snapshot;

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 1)]
struct Position {
    x: f32,
    y: f32,
}

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 2)]
struct Entity {
    id: u64,
    #[alkahest(version = 1)]
    position: Position,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let output = args.next().ok_or("usage: export OUTPUT.json")?;
    if args.next().is_some() {
        return Err("usage: export OUTPUT.json".into());
    }
    let mut snapshot = Snapshot::new();
    // Register reusable children independently so a parent bump cannot hide their changes.
    snapshot.add::<Position, 1>("game.position")?;
    snapshot.add::<Entity, 2>("game.entity")?;
    fs::write(output, snapshot.to_json()?)?;
    Ok(())
}
