use std::{env, ffi::CString};

use fontconfig::{self, Fontconfig, FontconfigError};

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

    let mut objects = fontconfig::ObjectSet::new(&fc)?;
    objects.add(fontconfig::FC_FAMILY)?;
    objects.add(fontconfig::FC_FILE)?;
    objects.add(fontconfig::FC_FONTFORMAT)?;
    objects.add(fontconfig::FC_INDEX)?;
    objects.add(fontconfig::FC_STYLE)?;

    let fontset = fontconfig::list_fonts(&pattern, Some(&objects))?;

    for pattern in fontset.iter() {
        let family = pattern.get_string(fontconfig::FC_FAMILY)?;
        let style = pattern.get_string(fontconfig::FC_STYLE)?;

        println!(
            "{}[{}] ({}): {}:style={}",
            pattern.filename()?,
            pattern.face_index()?,
            pattern
                .format()
                .map(|f| f.to_string())
                .unwrap_or_else(|_| String::from("Unknown")),
            family,
            style
        )
    }
    Ok(())
}
