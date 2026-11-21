//! What the generator will write from the Process ethos of the Tally Nexus,
//! once it reads a Nexus's roots together: the gifts' types, each process and
//! what it takes, and the enums over them. Written by hand until then; every
//! path names core or alloc, so it builds with no_std.

use alloc::vec::Vec;

use process::{Givable, Giving, Ownable, Owner, Processable, Unwindable};

use crate::{Add, Addend, Addition, Total};

macro_rules! generated {
    ($($item:item)*) => { $(
        #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
        #[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
        $item
    )* };
}

generated! {
    // The outputs given, each a type of its own
    pub struct Summed(pub Total);
    pub struct Overflowed;

    // Process
    pub struct SumProcess { pub addend_vector: Vec<Addend> }
    pub struct AddendProcess { pub addend: Addend }
    pub enum SumTaken { Addition(Addition) }
    pub enum AddendTaken { Addition(Addition) }

    // The enums over the Nexus's processes
    pub enum Process { Sum(SumProcess), Addend(AddendProcess) }
    pub enum Gift { Addend(Addend), Add(Add), Summed(Summed), Overflowed }
    pub enum Taken { Addition(Addition) }
    // One variant for each process, named for it; a process that ends one way carries nothing.
    pub enum Outcome { Sum, Addend(Addition), Unwound }
}

macro_rules! givable {
    ($($gift:ident => $returned:ty),* $(,)?) => { $(
        impl Givable for $gift { type Returned = $returned; }
        impl From<$gift> for Gift { fn from(gift: $gift) -> Gift { Gift::$gift(gift) } }
    )* };
}

// An output that carries nothing is a variant of the Gift that carries nothing.
impl Givable for Overflowed { type Returned = (); }
impl From<Overflowed> for Gift { fn from(Overflowed: Overflowed) -> Gift { Gift::Overflowed } }

givable! {
    // A process given: what returns for it is its outcome, at its end.
    Addend => Addition,
    Add => Addition,
    Summed => (),
}

impl Unwindable for Outcome {
    fn unwound() -> Self {
        Outcome::Unwound
    }
}

impl Ownable for Process {
    fn owner(&self) -> Owner {
        match self {
            Process::Sum(_) | Process::Addend(_) => Owner::Signal,
        }
    }
}

impl Processable for Process {
    type Taken = Taken;
    type Gift = Gift;
    type Outcome = Outcome;

    fn begin<N: Giving<Gift>>(&mut self, giving: N) -> Option<Outcome> {
        match self {
            Process::Sum(sum) => sum.begin(giving).map(|()| Outcome::Sum),
            Process::Addend(addend) => addend.begin(giving).map(Outcome::Addend),
        }
    }

    fn take<N: Giving<Gift>>(&mut self, taken: Taken, giving: N) -> Option<Outcome> {
        match (self, taken) {
            (Process::Sum(sum), Taken::Addition(addition)) => sum.take(SumTaken::Addition(addition), giving).map(|()| Outcome::Sum),
            (Process::Addend(addend), Taken::Addition(addition)) => addend.take(AddendTaken::Addition(addition), giving).map(Outcome::Addend),
        }
    }
}
