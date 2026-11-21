//! The Tally Nexus's whole trace, for each thing the process manager does:
//! a birth under an owner, a gift carried, an answer taken, an end returned
//! to the owner, an end from outside unwound the last born first, and the
//! memory letting go of what an ended process asked.

use process::Moment;
use tally_nexus::{Step, run, written};
use datom_codec::{Actualizing, Budget, Potential};
use protos::ReaderBudget;

fn steps(datom: &str) -> Vec<Step> {
    let mut budget = Budget { remaining: 4_096, reader: ReaderBudget::try_from(4_096).expect("a reader budget"), depth: 0, maximum_depth: 64 };
    Potential::<Vec<Step>>::from(datom).actualize(&mut budget).expect("the steps")
}

/// Each entry on one line, its whitespace collapsed.
fn traced(datom: &str) -> Vec<String> {
    written(&run(steps(datom), Moment(0))).iter().map(|entry| entry.split_whitespace().collect::<Vec<_>>().join(" ")).collect()
}

#[test]
fn a_sum_bears_an_addend_for_each_number_and_answers_the_total_when_the_last_has_ended() {
    assert_eq!(
        traced("[ Called.Sum.[ 1 2 3 ] ]"),
        [
            "Started.0",
            "Born.{ { 1 [] } Sum.{ [ 1 2 3 ] } }",
            "Gave.{ { 1 [] } Addend.1 }",
            "Gave.{ { 1 [] } Addend.2 }",
            "Gave.{ { 1 [] } Addend.3 }",
            "Born.{ { 1 [ 1 ] } Addend.{ 1 } }",
            "Gave.{ { 1 [ 1 ] } Add.1 }",
            "Born.{ { 1 [ 2 ] } Addend.{ 2 } }",
            "Gave.{ { 1 [ 2 ] } Add.2 }",
            "Born.{ { 1 [ 3 ] } Addend.{ 3 } }",
            "Gave.{ { 1 [ 3 ] } Add.3 }",
            "Took.{ { 1 [ 1 ] } Addition.Added.1 }",
            "Ended.{ { 1 [ 1 ] } Addend.Added.1 }",
            "Took.{ { 1 [] } Addition.Added.1 }",
            "Took.{ { 1 [ 2 ] } Addition.Added.3 }",
            "Ended.{ { 1 [ 2 ] } Addend.Added.3 }",
            "Took.{ { 1 [] } Addition.Added.3 }",
            "Took.{ { 1 [ 3 ] } Addition.Added.6 }",
            "Ended.{ { 1 [ 3 ] } Addend.Added.6 }",
            "Took.{ { 1 [] } Addition.Added.6 }",
            "Gave.{ { 1 [] } Summed.6 }",
            "Ended.{ { 1 [] } Sum }",
        ]
    );
}

#[test]
fn a_connection_closed_while_its_sum_adds_unwinds_the_addends_the_last_born_first_and_nothing_of_it_is_kept() {
    let trace = traced("[ Called.Sum.[ 1 2 3 ] Called.Sum.[ 4 5 ] Closed.2 ]");
    let ends: Vec<&str> = trace.iter().map(String::as_str).filter(|entry| entry.starts_with("Ended")).collect();
    assert_eq!(
        ends,
        [
            "Ended.{ { 2 [ 2 ] } Unwound }",
            "Ended.{ { 2 [ 1 ] } Unwound }",
            "Ended.{ { 2 [] } Unwound }",
            "Ended.{ { 1 [ 1 ] } Addend.Added.1 }",
            "Ended.{ { 1 [ 2 ] } Addend.Added.3 }",
            "Ended.{ { 1 [ 3 ] } Addend.Added.6 }",
            "Ended.{ { 1 [] } Sum }",
        ]
    );
    assert!(trace.contains(&"Gave.{ { 1 [] } Summed.6 }".to_owned()), "the first sum's total holds nothing of the second");
    assert!(!trace.iter().any(|entry| entry.starts_with("Took.{ { 2")), "nothing is taken by an unwound process");
}

#[test]
fn an_overflow_answers_overflowed_and_unwinds_the_addend_still_living() {
    let trace = traced("[ Called.Sum.[ 9223372036854775807 1 2 ] ]");
    assert_eq!(
        trace[trace.len() - 6..],
        [
            "Took.{ { 1 [ 2 ] } Addition.Overflowed }",
            "Ended.{ { 1 [ 2 ] } Addend.Overflowed }",
            "Took.{ { 1 [] } Addition.Overflowed }",
            "Gave.{ { 1 [] } Overflowed }",
            "Ended.{ { 1 [ 3 ] } Unwound }",
            "Ended.{ { 1 [] } Sum }",
        ]
    );
}
