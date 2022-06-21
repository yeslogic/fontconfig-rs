use std::ffi::CString;
use std::process;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(name = "fc-list", about = "List fonts matching")]
struct Opts {
    /// display entire font pattern verbosely
    #[clap(short, long, action)]
    verbose: bool,
    /// display font pattern briefly
    #[clap(short, long, action)]
    brief: bool,
    /// use the given output format
    #[clap(short = 'f', value_parser, long = "format", value_name = "FORMAT")]
    format: Option<String>,

    /// suppress all normal output, exit 1 if no fonts matched
    #[clap(short, long, action)]
    quiet: bool,

    /// display font config version and exit
    #[clap(short = 'V', long = "version", action)]
    version: bool,

    /// pattern
    #[clap(value_name = "pattern", value_parser)]
    pattern: Option<String>,

    /// element ...
    #[clap(value_name = "element", value_parser)]
    elements: Vec<String>,
}

fn main() {
    let opts = Opts::parse();
    if opts.version {
        let version = fontconfig::version();
        println!("fontconfig version {}", version);
        return;
    }

    let mut os = None;

    let pat = if let Some(ref pattern) = opts.pattern {
        let pat: fontconfig::OwnedPattern = pattern.parse().expect("Unable to parse the pattern");
        if !opts.elements.is_empty() {
            let mut objectset = fontconfig::ObjectSet::new();
            for element in opts.elements {
                objectset.add(&CString::new(element.to_string()).unwrap());
            }
            os.replace(objectset);
        }
        pat
    } else {
        fontconfig::OwnedPattern::new()
    };

    if opts.quiet && os.is_none() {
        os.replace(fontconfig::ObjectSet::new());
    }

    if !opts.verbose && !opts.brief && opts.format.is_none() && os.is_none() {
        let objectset = fontconfig::ObjectSet::build(&[
            fontconfig::FC_FAMILY.as_cstr(),
            fontconfig::FC_STYLE.as_cstr(),
            fontconfig::FC_FILE.as_cstr(),
        ]);
        os.replace(objectset);
    }

    let format = if let Some(fmt) = opts.format.clone() {
        CString::new(fmt).unwrap()
    } else {
        CString::new("%{=fclist}".to_string()).unwrap()
    };

    let mut config = fontconfig::FontConfig::default();
    let mut fs = pat.font_list(&mut config, os.as_mut());

    if !opts.quiet && !fs.is_empty() {
        for font in fs.iter_mut() {
            if opts.verbose || opts.brief {
                if opts.brief {
                    font.del(fontconfig::FC_CHARSET.as_cstr());
                    font.del(fontconfig::FC_LANG.as_cstr());
                }
                font.print();
            } else if let Some(fmt) = font.format(&format) {
                println!("{}", fmt.to_string_lossy());
            } else {
                eprintln!("Unable to format font");
            }
        }
    }

    if opts.quiet {
        process::exit(if fs.is_empty() { 1 } else { 0 })
    }
}
