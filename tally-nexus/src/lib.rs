//! The Tally Nexus's world, around the process manager: the signal process,
//! which takes each call and closes each connection; the memory process, which
//! keeps the tally and makes one change at a time; and the queue they share.
//! Calls are taken first; the memory makes its next change only when nothing
//! else waits, so a connection closed while its sum is adding shows the unwind.

use std::collections::{BTreeMap, VecDeque};

use datom_codec::{Composing, Datomizable};
use process::{Arrival, Ended, Gave, Given, Identity, Living, Moment, Path, ProcessManager, Root, Roots, Trace, Traced, Tracing};
use tally_process::*;

/// What reaches the Tally Nexus from outside: a call, or the closing of the
/// connection of the call drawn under that root.
#[derive(Clone, Debug, PartialEq, Eq, Datomizable, Composing)]
pub enum Step {
    Called(Input),
    Closed(Root),
}

struct World {
    queue: VecDeque<Arrival<Process>>,
    changes: VecDeque<(Identity, Add)>,
    tally: Tally,
    roots: Roots,
    positions: BTreeMap<Identity, i64>,
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
            trace: Vec::new(),
        }
    }

    fn step(&mut self, step: Step) {
        match step {
            Step::Called(Input::Sum(addend_vector)) => {
                let identity = self.roots.draw();
                self.queue.push_back(Arrival::Born(Living { identity, process: Process::Sum(SumProcess { addend_vector }) }));
            }
            Step::Closed(root) => {
                self.queue.push_back(Arrival::Ended(Ended { identity: Identity { root, path: Path(Vec::new()) }, outcome: Outcome::Unwound }));
            }
        }
    }

    /// The memory process: one change, kept, and its answer given back.
    fn change(&mut self, identity: Identity, Add(addend): Add) {
        let addition = match self.tally.total.0.checked_add(addend) {
            Some(total) => {
                self.tally = Tally { total: Total(total), count: Count(self.tally.count.0 + 1) };
                Addition::Added(Total(total))
            }
            None => Addition::Overflowed,
        };
        self.queue.push_back(Arrival::Given(Given { identity, taken: Taken::Addition(addition) }));
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
            // The signal process writes the answer back on the caller's connection.
            Gift::Summed(_) | Gift::Overflowed => {}
        }
    }

    /// An end is given to the owner, which takes the outcome; the memory lets go
    /// of what was asked under that identity.
    fn ended(&mut self, Ended { identity, outcome }: Ended<Outcome>) {
        self.changes.retain(|(asked, _)| *asked != identity);
        self.positions.remove(&identity);
        if let (Outcome::Addend(addition), Some(owner)) = (outcome, identity.owner()) {
            self.queue.push_back(Arrival::Given(Given { identity: owner, taken: Taken::Addition(addition) }));
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

/// Runs the steps to their end and gives the whole trace, Started first.
pub fn run(steps: Vec<Step>, moment: Moment) -> Vec<Traced<Process>> {
    let mut world = World::new();
    world.trace.push(Trace::Started(moment));
    let mut manager = ProcessManager::default();
    let mut steps = steps.into_iter();
    loop {
        if let Some(arrival) = world.queue.pop_front() {
            manager.arrive(arrival, &mut world);
        } else if let Some(step) = steps.next() {
            world.step(step);
        } else if let Some((identity, add)) = world.changes.pop_front() {
            world.change(identity, add);
        } else {
            break;
        }
    }
    manager.end_all(&mut world);
    world.trace
}

/// One line of datom for each entry of the trace.
pub fn written(trace: &[Traced<Process>]) -> Vec<String> {
    use protos::{Protosizable, Textualizable};
    trace.iter().map(|entry| entry.datomize(vec![]).protosize().textualize()).collect()
}
