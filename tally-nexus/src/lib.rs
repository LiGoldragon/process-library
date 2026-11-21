//! The Tally Nexus's world, around the process manager: the signal process,
//! which takes each call and closes each connection; the memory process, which
//! keeps the tally and makes one change at a time; and the queue they share.
//! Calls are taken first; the memory makes its next change only when nothing
//! else waits, so a connection closed while its sum is adding shows what a close
//! ends: nothing of a call's process, which lives until it yields its outcome;
//! what it then gives its caller is let go.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use datom_codec::{Composing, Datomizable};
use process::{Arrival, Caller, Ended, Gave, Given, Identity, Living, Moment, ProcessManager, Roots, Root, Trace, Traced, Tracing};
use tally_process::*;

/// A call: its input, and who made it.
#[derive(Clone, Debug, PartialEq, Eq, Datomizable, Composing)]
pub struct Call {
    pub caller: Caller,
    pub input: Input,
}

/// What reaches the Tally Nexus from outside: a call, or the closing of the
/// connection of the call drawn under that root; and a wait, for the memory to
/// have made every change asked of it before the next step comes, so that a
/// connection can close after its answer.
#[derive(Clone, Debug, PartialEq, Eq, Datomizable, Composing)]
pub enum Step {
    Called(Call),
    Closed(Root),
    Settled,
}

/// What a run came to: the whole trace, Started first, and each answer the
/// signal process wrote back on a connection that was still open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub trace: Vec<Traced<Process>>,
    pub answered: Vec<(Root, Gift)>,
}

struct World {
    queue: VecDeque<Arrival<Process>>,
    changes: VecDeque<(Identity, Add)>,
    tally: Tally,
    roots: Roots,
    positions: BTreeMap<Identity, i64>,
    closed: BTreeSet<Root>,
    settling: bool,
    answered: Vec<(Root, Gift)>,
    trace: Vec<Traced<Process>>,
}

impl World {
    fn new() -> World {
        World {
            queue: VecDeque::new(),
            changes: VecDeque::new(),
            tally: Tally { total: Total(0), count: Count(0) },
            roots: Roots::default(),
            positions: BTreeMap::new(),
            closed: BTreeSet::new(),
            settling: false,
            answered: Vec::new(),
            trace: Vec::new(),
        }
    }

    fn step(&mut self, step: Step) {
        match step {
            Step::Called(Call { caller, input: Input::Sum(integer_vector) }) => {
                let identity = self.roots.draw();
                self.queue.push_back(Arrival::Born(Living { identity, process: Process::Sum(SumProcess { caller, integer_vector }) }));
            }
            // A close after the answer ends nothing, and so does a close before it: the call's
            // process lives until it yields its outcome. The connection is shut, and what the
            // process gives its caller from now on is let go.
            Step::Closed(root) => {
                self.closed.insert(root);
            }
            Step::Settled => self.settling = true,
        }
    }

    /// The memory process: one change, kept, and what it came to given back.
    fn change(&mut self, identity: Identity, Add(addend): Add) {
        let addition = match self.tally.total.0.checked_add(addend) {
            Some(total) => {
                self.tally = Tally { total: Total(total), count: Count(self.tally.count.0 + 1) };
                Addition::Added(Total(total))
            }
            None => Addition::Overflowed,
        };
        self.queue.push_back(Arrival::Given(Given { identity, taken: Taken::Addition(Ok(addition)) }));
    }

    /// A process given is born under its giver: the giver's Root, its Path and the next position.
    fn bear(&mut self, owner: &Identity, addend: Addend) {
        let position = self.positions.entry(owner.clone()).or_insert(0);
        *position += 1;
        let identity = owner.owned(*position);
        self.queue.push_back(Arrival::Born(Living { identity, process: Process::Addend(AddendProcess { addend }) }));
    }

    fn carry(&mut self, Gave { identity, gift }: Gave<Gift>) {
        match gift {
            Gift::Addend(addend) => self.bear(&identity, addend),
            Gift::Add(add) => self.changes.push_back((identity, add)),
            // The signal process writes the answer back on the caller's connection, where it is still open.
            Gift::Summed(_) | Gift::Overflowed => {
                if !self.closed.contains(&identity.root) {
                    self.answered.push((identity.root, gift));
                }
            }
        }
    }

    /// An end is given to the owner, which takes the outcome; the memory lets go
    /// of what was asked under that identity.
    fn ended(&mut self, Ended { identity, outcome }: Ended<Outcome>) {
        self.changes.retain(|(asked, _)| *asked != identity);
        self.positions.remove(&identity);
        if let (Outcome::Addend(adding), Some(owner)) = (outcome, identity.owner()) {
            self.queue.push_back(Arrival::Given(Given { identity: owner, taken: Taken::Adding(adding) }));
        }
    }
}

impl Tracing<Process> for World {
    fn trace(&mut self, trace: Traced<Process>) {
        self.trace.push(trace.clone());
        match trace {
            Trace::Gave(gave) => self.carry(gave),
            Trace::Ended(ended) => self.ended(ended),
            Trace::Started(_) | Trace::Born(_) | Trace::Took(_) => {}
        }
    }
}

/// Runs the steps to their end and gives what it came to.
pub fn run(steps: Vec<Step>, moment: Moment) -> Run {
    let mut world = World::new();
    world.trace.push(Trace::Started(moment));
    let mut manager = ProcessManager::default();
    let mut steps = steps.into_iter();
    loop {
        if let Some(arrival) = world.queue.pop_front() {
            manager.arrive(arrival, &mut world);
        } else if world.settling {
            // Nothing else waits, so the memory makes its next change; the wait is over when it has none.
            match world.changes.pop_front() {
                Some((identity, add)) => world.change(identity, add),
                None => world.settling = false,
            }
        } else if let Some(step) = steps.next() {
            world.step(step);
        } else if let Some((identity, add)) = world.changes.pop_front() {
            world.change(identity, add);
        } else {
            break;
        }
    }
    manager.end_all(&mut world);
    Run { trace: world.trace, answered: world.answered }
}

/// One line of datom for each entry of the trace.
pub fn written(trace: &[Traced<Process>]) -> Vec<String> {
    use protos::{Protosizable, Textualizable};
    trace.iter().map(|entry| entry.datomize(vec![]).protosize().textualize()).collect()
}
