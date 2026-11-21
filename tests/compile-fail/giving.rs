// Both cases have the same gift and envelope. Only the concrete giver changes.
use process::Giving;
use tally_process::memory::Add;

#[cfg(not(wrong_process))]
type Body = tally_process::AddendProcess;
#[cfg(wrong_process)]
type Body = tally_process::SumProcess;

pub fn give_add(mut giving: impl Giving<Body>) {
    giving.give(Add(1));
}
