use std::ffi::{CStr, CString};
use std::str::FromStr;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
/// List best font matching [pattern]
struct Opts {
    #[clap(
        short = 's',
        long = "sort",
        action,
        help = "display sorted list of matches"
    )]
    sort: bool,
    /// display unpruned sorted list of matches
    #[clap(short = 'a', long = "all", action)]
    all: bool,
    /// display entire font pattern verbosely
    #[clap(short = 'v', long = "verbose", action)]
    verbose: bool,
    /// display entire font pattern briefly
    #[clap(short = 'b', long = "brief", action)]
    brief: bool,

    /// use the given output format
    #[clap(short = 'f', value_parser, long = "format", value_name = "FORMAT")]
    format: Option<String>,

    /// display font config version and exit
    #[clap(short = 'V', long = "version", action)]
    version: bool,

    /// pattern
    #[clap(value_parser, value_name = "PATTERN")]
    pattern: Option<String>,

    #[clap(value_parser, value_name = "element")]
    elements: Vec<String>,
}

fn main() {
    let opts = Opts::parse();

    if opts.version {
        let version = fontconfig::version();
        let major = version / 10000;
        let version = version % 10000;
        let minor = version / 100;
        let revision = version % 100;
        println!("fontconfig version {}.{}.{}", major, minor, revision);
        return;
    }

    let mut os = None;

    let mut pat = if let Some(ref pattern) = opts.pattern {
        let pat = fontconfig::Pattern::from_str(pattern).expect("Unable to parse the pattern");
        if !opts.elements.is_empty() {
            let mut objects = fontconfig::ObjectSet::new();
            for element in opts.elements {
                objects.add(&CString::new(element).unwrap());
            }
            os.replace(objects);
        }
        pat
    } else {
        fontconfig::Pattern::new()
    };

    let mut config = fontconfig::FontConfig::default();
    config.substitute(&mut pat, fontconfig::MatchKind::Pattern);
    pat.default_substitute();

    let mut fontset = fontconfig::FontSet::new();

    if opts.sort || opts.all {
        let patterns = pat
            .font_sort(&mut config, !opts.all)
            .expect("No fonts installed on the system");

        for pattern in patterns.iter() {
            let pat = pat.render_prepare(&pattern);
            fontset.push(pat);
        }
    } else {
        fontset.push(pat.font_match(&mut config))
    }

    let fmt = opts
        .format
        .map(|fmt| CString::new(fmt).unwrap())
        .unwrap_or_else(|| {
            if os.is_some() {
                unsafe { CStr::from_bytes_with_nul_unchecked(b"%{=unparse}\0") }.to_owned()
            } else {
                unsafe { CStr::from_bytes_with_nul_unchecked(b"%{=fcmatch}\0") }.to_owned()
            }
        });

    //
    for pattern in fontset.iter() {
        let mut font = pattern.filter(os.as_mut()).unwrap();
        if opts.verbose || opts.brief {
            if opts.brief {
                font.del(fontconfig::FC_CHARSET.as_cstr());
                font.del(fontconfig::FC_LANG.as_cstr());
            }
            pattern.print();
        } else {
            let s = pattern.format(&fmt);
            println!("{:?}", &s);
        }
    }
}
