use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(
    name = "fc-query",
    about = "Query font files and print resulting pattern(s)"
)]
struct Opts {
    /// display the INDEX face of each font file only
    #[clap(short, long, value_parser)]
    index: Option<isize>,
    /// display font pattern briefly
    #[clap(short, long, action)]
    brief: bool,
    /// use the given output format
    #[clap(short = 'f', value_parser, long = "format", value_name = "FORMAT")]
    format: Option<String>,

    /// display font config version and exit
    #[clap(short = 'V', long = "version", action)]
    version: bool,

    /// font files
    #[clap(value_name = "font-file", value_parser, min_values = 1)]
    files: Vec<std::path::PathBuf>,
}

fn main() {
    let opts = Opts::parse();
    if opts.version {
        let version = fontconfig::version();
        println!("fontconfig version {}", version);
        return;
    }

    let mut fs = fontconfig::FontSet::new();

    for file in opts.files {
        fs.freetype_query_all(&file, opts.index.unwrap_or(-1), None, None);
        // fprintf(
        //     stderr,
        //     _("Can't query face %u of font file %s\n"),
        //     id,
        //     argv[i],
        // );
    }
}
