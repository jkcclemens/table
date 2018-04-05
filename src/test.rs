use std_test::Bencher;

use untyped::Table;

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Test {
  number: u32,
  boolean: bool,
}

fn cleanup(s: &str) {
  // cleanup
  if let Err(e) = fs::remove_file(format!("{}.dat", s)) {
    eprintln!("could not delete {}.dat: {}", s, e);
  };

  if let Err(e) = fs::remove_file(format!("{}.idx", s)) {
    eprintln!("could not delete {}.idx: {}", s, e);
  };
}

#[test]
fn serialize() {
  // create our test struct to be serialized
  let test = Test {
    number: 5,
    boolean: true,
  };

  // these are the bytes that should be produced
  let ex_bytes = [0x92, 0x05, 0xc3];

  // open the table
  let mut table = Table::open("testerino").unwrap();

  // push the test struct onto the table
  table.push(&test).unwrap();

  // create vector
  let mut data = Vec::with_capacity(3);
  // open the data file
  let mut f = File::open("testerino.dat").unwrap();
  // read the entire data file into the vector
  let read = f.read_to_end(&mut data).unwrap();

  {
    // get the data
    let slice = &data[..read];

    // it should be equal to the bytes we expected
    assert_eq!(slice, ex_bytes);
  }

  // get the test struct back
  let de = table.pop().unwrap().unwrap();

  // make sure they're equal
  assert_eq!(test, de);

  // move to the beginning of the data file
  f.seek(SeekFrom::Start(0)).unwrap();

  // clear out the vector
  data.clear();

  // read again
  let read = f.read_to_end(&mut data).unwrap();

  // should be empty
  assert!(data[..read].is_empty());

  // cleanup
  cleanup("testerino");
}

#[bench]
fn append(b: &mut Bencher) {
  // open table
  let mut table = Table::open("testerino2").unwrap();
  // create our test struct
  let test = Test {
    number: 1_234_567,
    boolean: false,
  };
  // push it 500,000 times
  for _ in 0..500_000 {
    table.push(&test).unwrap();
  }
  // append more
  b.iter(|| table.push(&test));

  // cleanup
  cleanup("testerino2");
}

// test test::iter   ... bench: 1,830,532,810 ns/iter (+/- 888,871,538)
// aka about 0.9 to 2.7s for each 500,000 item loop
// this takes an obscenely long amount of time to run (~10 minutes)
#[bench]
#[ignore]
fn iter(b: &mut Bencher) {
  // open table
  let mut table = Table::open("testerino3").unwrap();
  // create our test struct
  let test = Test {
    number: 1_234_567,
    boolean: false,
  };
  // push it 500,000 times
  for _ in 0..500_000 {
    table.push(&test).unwrap();
  }
  // count items
  b.iter(|| table.iter::<Test>().count());

  // cleanup
  cleanup("testerino3");
}
