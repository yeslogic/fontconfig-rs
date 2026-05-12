use std::{env, ffi::CString};

use fontconfig::{self, FontSet, Fontconfig, FontconfigError};

fn main() -> Result<(), FontconfigError> {
    let fc = Fontconfig::new().expect("unable to init Fontconfig");

    let Some(family) = env::args().nth(1) else {
        eprintln!("Error: font family not specified");
        eprintln!(
            "Usage: {}: family",
            env::args().next().as_deref().unwrap_or("fc-list")
        );
        return Ok(());
    };
    let family = CString::new(family)?;

    let mut pattern = fontconfig::Pattern::new(&fc)?;
    pattern.add_string(fontconfig::FC_FAMILY, &family)?;

    let matched = pattern.font_match()?;
    let mut fontset = FontSet::new(&fc)?;
    fontset.add_pattern(matched)?;

    for pattern in fontset.iter() {
        println!(
            "{}[{}] ({}): {}, weight = {}, width = {}, slant = {}",
            pattern.filename()?,
            pattern.face_index()?,
            pattern
                .format()
                .map(|f| f.to_string())
                .unwrap_or_else(|_| String::from("Unknown")),
            pattern.name()?,
            pattern.weight()?,
            pattern.width()?,
            pattern.slant()?
        )
    }
    Ok(())
}
