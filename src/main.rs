use std::process;
use std::io::Cursor;
use std::io::prelude::*;
use clap::{App, Arg, SubCommand};

mod arsc;

use arsc::{Context, read_chunk};

fn main() {
    let matches = App::new("apktools-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .author("lds <lds2012@github.com>")
        .subcommand(SubCommand::with_name("packagename")
                    .about("get package name of apk")
                    .arg(Arg::with_name("apkfile").required(true).help("apk file"))
                )
        .get_matches();

    if let Some(opts) = matches.subcommand_matches("packagename") {
        let apkfile = opts.value_of("apkfile").expect("missing apkfile arg.");
        let apkfile = std::path::Path::new(apkfile);
        if !apkfile.exists() {
            eprintln!("{} file is not exists!", apkfile.display());
            process::exit(-1); 
        }

        let zipfile = std::fs::File::open(&apkfile).unwrap();
        let mut archive = zip::ZipArchive::new(zipfile).unwrap();
        let mut manifest_file = match archive.by_name("AndroidManifest.xml") {
            Ok(file) => file,
            Err(..) => {
                eprintln!("Can not find AndroidManifest.xml in {}!", apkfile.display());
                process::exit(-1); 
            }
        };

        let mut buffer: Vec<u8> = Vec::new();
        manifest_file.read_to_end(&mut buffer).unwrap();
        let mut cursor = Cursor::new(&buffer);

        let mut context = Context {
            strings_pool: Vec::new() 
        };
        read_chunk(&mut context, &mut cursor);
    }
}
