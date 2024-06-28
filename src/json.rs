use std::{fs::File, io::{self, BufReader}};

use serde::{Deserialize, Serialize};

pub fn read_json_file<T: for<'de> Deserialize<'de>>(file_path: &str) -> io::Result<T> {
  let file = File::open(file_path)?;
  let reader = BufReader::new(file);
  let data = serde_json::from_reader(reader)?;

  Ok(data)
}

pub fn write_json_file<T: Serialize>(file_path: &str, data: &T) -> io::Result<()> {
  let file = File::create(file_path)?;
  let writer = io::BufWriter::new(file);
  
  serde_json::to_writer_pretty(writer, data)?;
 
  Ok(())
}