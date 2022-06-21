use std::ffi::CString;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(name = "fc-pattern", about = "List best font matching [pattern]")]
struct Opts {
    /// perform config substitution on pattern
    #[clap(short, long, action)]
    config: bool,

    /// perform default substitution on pattern
    #[clap(short, long, action)]
    default: bool,

    /// use the given output format
    #[clap(short, long, value_parser)]
    format: Option<String>,

    /// display font config version and exit
    #[clap(short, long, action)]
    version: bool,

    /// pattern
    #[clap(value_name = "pattern", value_parser)]
    pattern: Option<String>,

    /// element ...
    #[clap(value_name = "element", value_parser)]
    elements: Vec<String>,
}

fn main() {
    let mut opts = Opts::parse();
    if opts.version {
        let version = fontconfig::version();
        eprintln!("fontconfig version {}", version);
        return;
    }

    let mut os = None;

    let mut pat: fontconfig::OwnedPattern = if let Some(ref pattern) = opts.pattern {
        if !opts.elements.is_empty() {
            let mut objectset = fontconfig::ObjectSet::new();
            for element in opts.elements {
                objectset.add(&CString::new(element.to_string()).unwrap());
            }
            os.replace(objectset);
        }
        pattern.parse().expect("Unable to parse the pattern")
    } else {
        fontconfig::OwnedPattern::new()
    };

    let mut config = fontconfig::FontConfig::default();

    if opts.default {
        pat.default_substitute();
    }

    if opts.config {
        config.substitute(&mut pat, fontconfig::MatchKind::Pattern);
    }

    if os.is_some() {
        pat = pat.filter(os.as_mut()).unwrap();
    }

    if let Some(fmt) = opts.format.take() {
        if let Some(s) = pat.format(&CString::new(fmt).unwrap()) {
            println!("{}", s.to_string_lossy());
        }
    } else {
        pat.print();
    }
}
