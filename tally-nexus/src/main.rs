//! tally-nexus '<steps>': runs the steps, a vector of Called.{ Caller Input }
//! and Closed.<Root>, and writes the trace as datom, one line for each birth,
//! gift, taking and end.

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use datom_codec::{Actualizing, Budget, Potential};
use process::Moment;
use protos::ReaderBudget;
use tally_nexus::{Step, run, written};

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let (Some(datom), None) = (arguments.next(), arguments.next()) else {
        eprintln!("Usage");
        return ExitCode::from(2);
    };
    let mut budget = Budget { remaining: 4_096, reader: ReaderBudget::try_from(4_096).expect("a reader budget"), depth: 0, maximum_depth: 64 };
    let steps: Vec<Step> = match Potential::<Vec<Step>>::from(datom.as_str()).actualize(&mut budget) {
        Ok(steps) => steps,
        Err(error) => {
            eprintln!("{error:?}");
            return ExitCode::from(2);
        }
    };
    let moment = SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_nanos() as i64).unwrap_or(0);
    for line in written(&run(steps, Moment(moment)).trace) {
        println!("{line}");
    }
    ExitCode::SUCCESS
}
