#[macro_use]
extern crate clap;
extern crate bam;
extern crate varlink;

use std::fs::File;
use std::{fmt, io};

use bam::{IndexedReader, Region};
use clap::{AppSettings, Clap};
use varlink::{ValidateOptions, Variant, VariantValidate};

#[derive(Debug, Default)]
struct Link {
    both: u32,
    first: u32,
    second: u32,
    neither: u32,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\n  \"both\": {},\n  \"first\": {},\n  \"second\": {},\n  \"neither\": {}\n}}",
            self.both, self.first, self.second, self.neither
        )
    }
}

#[derive(Clap)]
#[clap(name = crate_name!(), version = crate_version!(), author = crate_authors!(), about = crate_description!())]
#[clap(setting = AppSettings::ArgRequiredElseHelp)]
struct Opts {
    #[clap(short = '1', long, about = "First variant, in HGVS format.")]
    first: String,
    #[clap(short = '2', long, about = "Second variant, in HGVS format.")]
    second: String,
    #[clap(short, long, about = "Bam file path.")]
    bam: String,
    #[clap(
        long,
        about = "When *merge* variant of the target exists, do not count read as a support."
    )]
    merge: bool,
}

fn get_merge_region(
    bam: &IndexedReader<File>,
    var1: &Variant,
    var2: &Variant,
) -> Result<Region, io::Error> {
    if var1.contig() != var2.contig() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Inconsistent contig: {} and {}.",
                var1.contig(),
                var2.contig()
            ),
        ));
    };
    let ref_id = if let Some(v) = bam.header().reference_id(var1.contig()) {
        v
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("No such id found for contig: {}.", var1.contig()),
        ));
    };
    Ok(Region::new(
        ref_id,
        u32::min(*var1.start(), *var2.start()),
        u32::max(*var1.end(), *var2.end()) + 1,
    ))
}

fn main() -> Result<(), io::Error> {
    let opts: Opts = Opts::parse();
    let mut link = Link::default();
    let first = Variant::parse(&opts.first)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}", e)))?;
    let second = Variant::parse(&opts.second)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}", e)))?;
    let mut reader = IndexedReader::from_path(&opts.bam)?;
    let region = get_merge_region(&reader, &first, &second)?;
    let mut options = ValidateOptions::default();
    options.set_merge(opts.merge);
    reader.fetch(&region)?.try_for_each(|rec| {
        let rec = rec?;
        let f1 = if let Some(v) = rec.validate(&first, &options) {
            v
        } else {
            return Ok(());
        };
        let f2 = if let Some(v) = rec.validate(&second, &options) {
            v
        } else {
            return Ok(());
        };
        if f1 && f2 {
            link.both += 1;
        } else if f1 {
            link.first += 1;
        } else if f2 {
            link.second += 1;
        } else {
            link.neither += 1
        };
        Ok::<(), io::Error>(())
    })?;
    println!("{}", link);
    Ok(())
}
