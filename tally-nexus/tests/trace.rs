//! The Tally Nexus's whole trace, for each thing the process manager does:
//! a birth under an owner, a gift carried, an answer taken, an end returned
//! to the owner, and the memory letting go of what an ended process asked; and
//! what a close of a connection ends: nothing of a call's process, which lives
//! until it yields its outcome, whose gifts to its caller after the close are
//! let go, as a close after the answer is.

use datom_codec::{Actualizing, Budget, Potential};
use process::Moment;
use protos::ReaderBudget;
use tally_nexus::{Run, Step, run, written};

fn steps(datom: &str) -> Vec<Step> {
    let mut budget = Budget { remaining: 4_096, reader: ReaderBudget::try_from(4_096).expect("a reader budget"), depth: 0, maximum_depth: 64 };
    Potential::<Vec<Step>>::from(datom).actualize(&mut budget).expect("the steps")
}

fn ran(datom: &str) -> Run {
    run(steps(datom), Moment(0))
}

/// Each entry on one line, its whitespace collapsed.
fn traced(datom: &str) -> Vec<String> {
    written(&ran(datom).trace).iter().map(|entry| entry.split_whitespace().collect::<Vec<_>>().join(" ")).collect()
}

#[test]
fn a_sum_bears_an_addend_for_each_number_and_answers_the_total_when_the_last_has_ended() {
    assert_eq!(
        traced("[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } ]"),
        [
            "Started.0",
            "Born.{ { 1 [] } Sum.{ { 100 1000 } [ 1 2 3 ] } }",
            "Gave.{ { 1 [] } Addend.1 }",
            "Gave.{ { 1 [] } Addend.2 }",
            "Gave.{ { 1 [] } Addend.3 }",
            "Born.{ { 1 [ 1 ] } Addend.{ 1 } }",
            "Gave.{ { 1 [ 1 ] } Add.1 }",
            "Born.{ { 1 [ 2 ] } Addend.{ 2 } }",
            "Gave.{ { 1 [ 2 ] } Add.2 }",
            "Born.{ { 1 [ 3 ] } Addend.{ 3 } }",
            "Gave.{ { 1 [ 3 ] } Add.3 }",
            "Took.{ { 1 [ 1 ] } Addition.Ok.Added.1 }",
            "Ended.{ { 1 [ 1 ] } Addend.Added.1 }",
            "Took.{ { 1 [] } Adding.Added.1 }",
            "Took.{ { 1 [ 2 ] } Addition.Ok.Added.3 }",
            "Ended.{ { 1 [ 2 ] } Addend.Added.3 }",
            "Took.{ { 1 [] } Adding.Added.3 }",
            "Took.{ { 1 [ 3 ] } Addition.Ok.Added.6 }",
            "Ended.{ { 1 [ 3 ] } Addend.Added.6 }",
            "Took.{ { 1 [] } Adding.Added.6 }",
            "Gave.{ { 1 [] } Summed.6 }",
            "Ended.{ { 1 [] } Sum.Answered }",
        ]
    );
}

#[test]
fn the_answer_is_written_on_the_connection_while_it_is_open() {
    let run = ran("[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } ]");
    assert_eq!(run.answered.len(), 1);
    assert_eq!(run.answered[0].0, process::Root(1));
}

#[test]
fn a_close_after_the_answer_ends_nothing() {
    let answered = traced("[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } ]");
    assert_eq!(traced("[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } Settled Closed.1 ]"), answered);
    assert_eq!(ran("[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } Settled Closed.1 ]").answered.len(), 1);
    // A close before the answer is the close that shuts the connection: the answer is let go.
    assert_eq!(ran("[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } Closed.1 ]").answered.len(), 0);
}

#[test]
fn a_connection_closed_while_its_sum_adds_ends_nothing_the_sum_lives_to_its_outcome_and_its_answer_is_let_go() {
    let run = ran("[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } Called.{ { 100 1000 } Sum.[ 4 5 ] } Closed.2 ]");
    let trace: Vec<String> = written(&run.trace).iter().map(|entry| entry.split_whitespace().collect::<Vec<_>>().join(" ")).collect();
    let ends: Vec<&str> = trace.iter().map(String::as_str).filter(|entry| entry.starts_with("Ended")).collect();
    assert_eq!(
        ends,
        [
            "Ended.{ { 1 [ 1 ] } Addend.Added.1 }",
            "Ended.{ { 1 [ 2 ] } Addend.Added.3 }",
            "Ended.{ { 1 [ 3 ] } Addend.Added.6 }",
            "Ended.{ { 1 [] } Sum.Answered }",
            "Ended.{ { 2 [ 1 ] } Addend.Added.10 }",
            "Ended.{ { 2 [ 2 ] } Addend.Added.15 }",
            "Ended.{ { 2 [] } Sum.Answered }",
        ]
    );
    // The second sum gave its total after its connection had closed, and it was let go.
    assert!(trace.contains(&"Gave.{ { 2 [] } Summed.15 }".to_owned()));
    assert_eq!(run.answered.len(), 1);
    assert_eq!(run.answered[0].0, process::Root(1));
    assert!(!trace.iter().any(|entry| entry.contains("Unwound")), "nothing was unwound by a close");
}

#[test]
fn an_overflow_answers_overflowed_and_unwinds_the_addend_still_living() {
    let run = ran("[ Called.{ { 100 1000 } Sum.[ 9223372036854775807 1 2 ] } ]");
    let trace: Vec<String> = written(&run.trace).iter().map(|entry| entry.split_whitespace().collect::<Vec<_>>().join(" ")).collect();
    assert_eq!(
        trace[trace.len() - 6..],
        [
            "Took.{ { 1 [ 2 ] } Addition.Ok.Overflowed }",
            "Ended.{ { 1 [ 2 ] } Addend.Refused }",
            "Took.{ { 1 [] } Adding.Refused }",
            "Gave.{ { 1 [] } Overflowed }",
            "Ended.{ { 1 [ 3 ] } Unwound }",
            "Ended.{ { 1 [] } Sum.Refused }",
        ]
    );
    assert_eq!(run.answered.len(), 1);
}
