extern crate csv;
use std::fs::File;
use std::path::Path;

pub fn read(filename : &Path) -> Vec<Vec<String>> {
    let mut lines : Vec<Vec<String>> = vec![];
    let mut reader = csv::Reader::from_file(filename)
        .unwrap()
        .has_headers(false); // has but keep them inside
    for row in reader.records().map(|r| r.unwrap()) {
        lines.push(row);
    }
    lines
}

pub fn write(lines: &Vec<Vec<String>>, filename : &Path) {
    let mut writer = csv::Writer::from_file(filename).unwrap();
    for record in lines.iter() {
        writer.encode(record).unwrap();
    }
}

// check if the file already exists
pub fn check(filename : &Path) -> bool {
    match File::open(filename) {
        Ok(_) => true,
        _ => false,
    }
}
