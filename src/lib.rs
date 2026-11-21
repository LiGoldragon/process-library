//! The process library: a process is an object at rest in the manager's table,
//! given one thing at a time, and it gives.
//!
//! Nothing here waits. What waits, the queue, the sockets and the disk, is the
//! world the manager is handed: the queue it iterates and the Tracing it
//! writes every birth, gift, taking and end into. The manager draws no
//! identity: the one that bears a process draws it.

// The text side, where datom is spoken, is built with std; the Nexus's build is not.
#![cfg_attr(not(feature = "datom"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicI64, Ordering};

#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Root(pub i64);

#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(pub Vec<i64>);

#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {
    pub root: Root,
    pub path: Path,
}

#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Living<P> {
    pub identity: Identity,
    pub process: P,
}

#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Given<T> {
    pub identity: Identity,
    pub taken: T,
}

#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Gave<G> {
    pub identity: Identity,
    pub gift: G,
}

#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Ended<O> {
    pub identity: Identity,
    pub outcome: O,
}

/// A moment, in nanoseconds since the Unix epoch.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moment(pub i64);

/// Every entry of a run's trace: its first is Started, the program's own start,
/// so a run is told from the one before it; identities are unique within a run.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Trace<P, G, T, O> {
    Started(Moment),
    Born(Living<P>),
    Gave(Gave<G>),
    Took(Given<T>),
    Ended(Ended<O>),
}

/// A process: it begins once, takes each thing that comes for it after, and
/// yields its outcome when it ends. None is rest.
pub trait Processable: Sized {
    type Taken;
    type Gift;
    type Outcome;
    fn begin<N: Giving<Self>>(&mut self, giving: N) -> Option<Self::Outcome>;
    fn take<N: Giving<Self>>(&mut self, taken: Self::Taken, giving: N) -> Option<Self::Outcome>;
}

/// What a process is handed to give with. To give is to move.
pub trait Giving<A: Processable> {
    fn give<N: Givable<A> + Into<A::Gift>>(&mut self, gift: N);
}

/// A thing declared to leave a particular process; Returned is what comes back for it.
pub trait Givable<A: Processable> {
    type Returned;
}

/// The dispatch boundary of the aggregate Process the manager holds.
/// Its implementation supplies each concrete Processable body with its typed giver.
pub trait Dispatchable: Sized {
    type Taken;
    type Gift;
    type Outcome;

    fn dispatch_begin(&mut self, receive: impl FnMut(Self::Gift)) -> Option<Self::Outcome>;
    fn dispatch_take(
        &mut self,
        taken: Self::Taken,
        receive: impl FnMut(Self::Gift),
    ) -> Option<Self::Outcome>;
}

/// Turns the manager's gift receiver into the giving capability of one concrete process.
pub struct Giver<A, Receive> {
    receive: Receive,
    process: PhantomData<fn() -> A>,
}

impl<A, Receive> From<Receive> for Giver<A, Receive>
where
    A: Processable,
    Receive: FnMut(A::Gift),
{
    fn from(receive: Receive) -> Self {
        Self { receive, process: PhantomData }
    }
}

impl<A, Receive> Giving<A> for Giver<A, Receive>
where
    A: Processable,
    Receive: FnMut(A::Gift),
{
    fn give<N: Givable<A> + Into<A::Gift>>(&mut self, gift: N) {
        (self.receive)(gift.into());
    }
}

/// The outcome of a process ended with its owner.
pub trait Unwindable {
    fn unwound() -> Self;
}

/// The process id of the peer of a connection, from its kernel credentials.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub i64);

/// The user id of the peer of a connection, from its kernel credentials.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(pub i64);

/// Who called: every input reaches a Nexus with its Caller, and a process born of
/// a call holds it.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Caller {
    pub process_id: ProcessId,
    pub user_id: UserId,
}

/// What returns to a process for a record kept.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Answer {
    Succeeded,
    Failed,
}

/// What returns to a process for a change that was not made: its `Result` is its answer or this.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unremembered {}

/// What returns to a process for an emission whose outcome was not observed:
/// its `Result` is its outcome or this. Unsubmitted: no translator listened, nothing
/// was done; Unanswered: the translator took it and the connection closed first, so
/// the call may have been made.
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Unobserved {
    Unsubmitted,
    Unanswered,
}

/// The two waiting sides of the library, outside the table: every root process
/// is owned by one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Signal,
    Memory,
}

/// A process is born of a call, owned by the signal process, or of a record,
/// owned by the memory process.
pub trait Ownable {
    fn owner(&self) -> Owner;
}

/// The one counter the signal process and the memory process draw a Root from.
#[derive(Debug, Default)]
pub struct Roots(AtomicI64);

impl Roots {
    pub fn draw(&self) -> Identity {
        Identity { root: Root(self.0.fetch_add(1, Ordering::Relaxed) + 1), path: Path(Vec::new()) }
    }
}

/// The trace of one Nexus's processes.
pub type Traced<P> = Trace<P, <P as Dispatchable>::Gift, <P as Dispatchable>::Taken, <P as Dispatchable>::Outcome>;

/// What the queue yields: a process born, its identity already drawn by the one
/// that bears it; a thing for a living one; or the end of one, which is taken
/// out of the table at that moment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arrival<P: Dispatchable> {
    Born(Living<P>),
    Given(Given<P::Taken>),
    Ended(Ended<P::Outcome>),
}

/// Where the trace goes: every gift is carried from here, every end announced.
pub trait Tracing<P: Dispatchable> {
    fn trace(&mut self, trace: Traced<P>);

    fn trace_gift(&mut self, identity: &Identity, gift: P::Gift) {
        self.trace(Trace::Gave(Gave { identity: identity.clone(), gift }));
    }
}

pub struct ProcessManager<P> {
    table: Vec<Living<P>>,
}

impl<P> Default for ProcessManager<P> {
    fn default() -> Self {
        Self { table: Vec::new() }
    }
}

impl<P> ProcessManager<P>
where
    P: Dispatchable + Clone,
    P::Taken: Clone,
    P::Outcome: Unwindable,
{
    /// The manager's one queue is any iterator of arrivals: its next waits for
    /// the next thing, and its end ends every process.
    pub fn run<Q: IntoIterator<Item = Arrival<P>>, W: Tracing<P>>(&mut self, queue: Q, world: &mut W)
    where
        P: Ownable,
    {
        for arrival in queue {
            self.arrive(arrival, world);
        }
        self.end_all(world);
    }

    pub fn arrive<W: Tracing<P>>(&mut self, arrival: Arrival<P>, world: &mut W) {
        match arrival {
            Arrival::Born(living) => self.bear(living, world),
            Arrival::Given(given) => self.hand(given, world),
            Arrival::Ended(ended) => {
                if self.table.iter().any(|living| living.identity == ended.identity) {
                    self.end(ended.identity, ended.outcome, world);
                }
            }
        }
    }

    /// Puts a process in the table and begins it. A process born under an
    /// owner that has ended is let go.
    pub fn bear<W: Tracing<P>>(&mut self, mut living: Living<P>, world: &mut W) {
        let owned = !living.identity.path.0.is_empty();
        if owned && !self.table.iter().any(|held| held.identity.owns(&living.identity)) {
            return;
        }
        world.trace(Trace::Born(living.clone()));
        let identity = &living.identity;
        let outcome = living.process.dispatch_begin(|gift| world.trace_gift(identity, gift));
        let identity = living.identity.clone();
        self.table.push(living);
        if let Some(outcome) = outcome {
            self.end(identity, outcome, world);
        }
    }

    /// Hands a thing to the process it is for; a thing for an ended process is let go.
    pub fn hand<W: Tracing<P>>(&mut self, given: Given<P::Taken>, world: &mut W) {
        let Some(living) = self.table.iter_mut().find(|living| living.identity == given.identity) else {
            return;
        };
        world.trace(Trace::Took(given.clone()));
        let identity = &living.identity;
        let outcome = living.process.dispatch_take(given.taken, |gift| world.trace_gift(identity, gift));
        if let Some(outcome) = outcome {
            self.end(given.identity, outcome, world);
        }
    }

    /// A reverse unwind: what the process owns ends first, the last born first.
    pub fn end<W: Tracing<P>>(&mut self, identity: Identity, outcome: P::Outcome, world: &mut W) {
        let owned: Vec<Identity> = self.owned_by(&identity).collect();
        for owned in owned.into_iter().rev() {
            self.end(owned, P::Outcome::unwound(), world);
        }
        self.table.retain(|living| living.identity != identity);
        world.trace(Trace::Ended(Ended { identity, outcome }));
    }

    /// Every living process: what the signal process owns, then what the
    /// memory process owns, each the last born first.
    pub fn end_all<W: Tracing<P>>(&mut self, world: &mut W)
    where
        P: Ownable,
    {
        for owner in [Owner::Signal, Owner::Memory] {
            let roots: Vec<Identity> = self
                .table
                .iter()
                .filter(|living| living.identity.path.0.is_empty() && living.process.owner() == owner)
                .map(|living| living.identity.clone())
                .collect();
            for root in roots.into_iter().rev() {
                self.end(root, P::Outcome::unwound(), world);
            }
        }
    }

    fn owned_by<'a>(&'a self, owner: &'a Identity) -> impl Iterator<Item = Identity> + 'a {
        self.table.iter().map(|living| &living.identity).filter(move |identity| owner.owns(identity)).cloned()
    }
}

impl Identity {
    /// The owner of a process is read from its identity: its Root, and its
    /// Path with one position more.
    pub fn owns(&self, other: &Identity) -> bool {
        other.root == self.root && other.path.0.len() == self.path.0.len() + 1 && other.path.0.starts_with(&self.path.0)
    }

    /// The identity a process bears a process under: its own Root, and its
    /// Path with the position after it.
    pub fn owned(&self, position: i64) -> Identity {
        let mut path = self.path.0.clone();
        path.push(position);
        Identity { root: self.root, path: Path(path) }
    }

    /// The process that owns this one; a root process is owned by the signal
    /// process or the memory process, outside the table.
    pub fn owner(&self) -> Option<Identity> {
        let (_, path) = self.path.0.split_last()?;
        Some(Identity { root: self.root, path: Path(path.to_vec()) })
    }
}
