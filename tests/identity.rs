//! The owner of a process is read from its identity, and a process born under
//! an owner that has ended is let go.

use process_library::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Rest;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Nothing;

impl Unwindable for Nothing {
    fn unwound() -> Self {
        Nothing
    }
}

impl Ownable for Rest {
    fn owner(&self) -> Owner {
        Owner::Signal
    }
}

impl Dispatchable for Rest {
    type Taken = Nothing;
    type Gift = Nothing;
    type Outcome = Nothing;
    fn dispatch_begin(&mut self, _receive: impl FnMut(Nothing)) -> Option<Nothing> {
        None
    }
    fn dispatch_take(&mut self, _taken: Nothing, _receive: impl FnMut(Nothing)) -> Option<Nothing> {
        None
    }
}

#[derive(Default)]
struct Kept(Vec<Traced<Rest>>);

impl Tracing<Rest> for Kept {
    fn trace(&mut self, trace: Traced<Rest>) {
        self.0.push(trace);
    }
}

#[test]
fn a_process_owned_at_a_position_is_owned_by_the_identity_it_was_borne_under() {
    let root = Roots::default().draw();
    let owned = root.owned(1).owned(2);
    assert_eq!(owned, Identity { root: Root(1), path: Path(vec![1, 2]) });
    assert_eq!(owned.owner(), Some(root.owned(1)));
    assert!(root.owned(1).owns(&owned));
    assert_eq!(root.owner(), None);
}

#[test]
fn a_process_born_under_an_owner_that_has_ended_is_let_go() {
    let mut manager = ProcessManager::default();
    let mut kept = Kept::default();
    let root = Roots::default().draw();
    manager.bear(Living { identity: root.clone(), process: Rest }, &mut kept);
    manager.arrive(Arrival::Ended(Ended { identity: root.clone(), outcome: Nothing }), &mut kept);
    manager.bear(Living { identity: root.owned(1), process: Rest }, &mut kept);
    assert_eq!(kept.0, [Trace::Born(Living { identity: root.clone(), process: Rest }), Trace::Ended(Ended { identity: root, outcome: Nothing })]);
}
