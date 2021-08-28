extern crate bam;
extern crate varlink;

use bam::BamReader;
use varlink::{ValidateOptions, Variant, VariantValidate};

#[test]
fn test_validate_insertion() {
    // MACH:453:FLOWCELL:1:1222:15483:21825
    let var = Variant::from_hgvs("1:144852532_144852533insCCC").unwrap();
    let options = ValidateOptions::default();
    let mut reader = BamReader::from_path("tests/test.1:144852532-144852632.bam", 0).unwrap();
    let rec = loop {
        let v = if let Some(r) = reader.next() {
            r
        } else {
            panic!("Record not found.")
        };
        let v = v.unwrap();
        if v.name() == b"MACH:453:FLOWCELL:1:1222:15483:21825" {
            break v;
        };
    };
    assert!(rec.validate(&var, &options).unwrap());
    let rec = loop {
        let v = if let Some(r) = reader.next() {
            r
        } else {
            panic!("Record not found.")
        };
        let v = v.unwrap();
        if v.name() == b"MACH:453:FLOWCELL:1:1176:28619:20243" && v.flag().first_in_pair() {
            break v;
        };
    };
    assert!(rec.validate(&var, &options).unwrap());
    assert!(!rec
        .validate(
            &Variant::from_hgvs("1:144852532_144852533insCCG").unwrap(),
            &options
        )
        .unwrap());
}

#[test]
fn test_validate_deletion() {
    // MACH:453:FLOWCELL:1:1208:9643:7138
    let var = Variant::from_hgvs("1:144852633_144852634del").unwrap();
    let options = ValidateOptions::default();
    let mut reader = BamReader::from_path("tests/test.1:144852532-144852632.bam", 0).unwrap();
    let rec = loop {
        let v = if let Some(r) = reader.next() {
            r
        } else {
            panic!("Record not found.")
        };
        let v = v.unwrap();
        if v.name() == b"MACH:453:FLOWCELL:1:1176:28619:20243" && v.flag().first_in_pair() {
            break v;
        };
    };
    assert!(rec.validate(&var, &options).unwrap());
    let rec = loop {
        let v = if let Some(r) = reader.next() {
            r
        } else {
            panic!("Record not found.")
        };
        let v = v.unwrap();
        if v.name() == b"MACH:453:FLOWCELL:1:1208:9643:7138" && v.flag().last_in_pair() {
            break v;
        };
    };
    assert!(rec.validate(&var, &options).unwrap());
    assert!(!rec
        .validate(
            &Variant::from_hgvs("1:144852633_144852635del").unwrap(),
            &options
        )
        .unwrap());
}

#[test]
fn test_validate_snv() {
    // MACH:453:FLOWCELL:1:2121:4255:24267
    let var = Variant::from_hgvs("1:144852545C>T").unwrap();
    let options = ValidateOptions::default();
    let mut reader = BamReader::from_path("tests/test.1:144852532-144852632.bam", 0).unwrap();
    let rec = loop {
        let v = if let Some(r) = reader.next() {
            r
        } else {
            panic!("Record not found.")
        };
        let v = v.unwrap();
        if v.name() == b"MACH:453:FLOWCELL:1:2121:4255:24267" {
            break v;
        };
    };
    assert!(rec.validate(&var, &options).unwrap());
    assert!(!rec
        .validate(&Variant::from_hgvs("1:144852545C>A").unwrap(), &options)
        .unwrap());
}

#[test]
fn test_validate_merge() {
    // MACH:453:FLOWCELL:1:2121:4255:24267
    let var1 = Variant::from_hgvs("1:144854597T>C").unwrap();
    let var2 = Variant::from_hgvs("1:144854598C>T").unwrap();
    let mut options = ValidateOptions::default();
    let mut reader = BamReader::from_path("tests/merge.1:144854597-144854608.bam", 0).unwrap();
    let rec = loop {
        let v = if let Some(r) = reader.next() {
            r
        } else {
            panic!("Record not found.")
        };
        let v = v.unwrap();
        if v.name() == b"MACH:277:FLOWCELL:1:2322:26856:36605" {
            break v;
        };
    };
    // Ignore read with *merge* variant.
    options.set_merge(true);
    assert!(!rec.validate(&var1, &options).unwrap());
    assert!(!rec.validate(&var2, &options).unwrap());
    // Include read with *merge* variant.
    options.set_merge(false);
    assert!(rec.validate(&var1, &options).unwrap());
    assert!(rec.validate(&var2, &options).unwrap());
}

#[test]
fn test_validate_multiple_snp() {
    let var1 = Variant::from_hgvs("1:144854047_144854048delinsTG").unwrap();
    let var2 = Variant::from_hgvs("1:144854049C>G").unwrap();
    let options = ValidateOptions::default();
    let mut reader = BamReader::from_path("tests/triple-snp.1:144854047-144854049.bam", 0).unwrap();
    let rec = loop {
        let v = if let Some(r) = reader.next() {
            r
        } else {
            panic!("Record not found.");
        };
        let v = v.unwrap();
        if v.name() == b"MACH:FLOWCELL:1:1537:20112:31093" {
            break v;
        };
    };
    assert!(rec.validate(&var1, &options).unwrap());
    assert!(rec.validate(&var2, &options).unwrap());
}
