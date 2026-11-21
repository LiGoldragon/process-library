# process-library

The process library of Process.4: a process is an object at rest in the
manager's table, given one thing at a time, and it gives. Nothing in it waits;
what waits, the queue, the sockets and the disk, is the world the manager is
handed. It builds with no operating system under it unless the `datom` feature
asks for the text side.

Lifted from `~/Code/claude-harness-nexus/process` at f0b425a, with two
additions: `Identity::owned`, the identity a process bears a process under,
and `Identity::owner`, read back from it; and, from 0.2.0, the standard types.

```toml
process = { package = "process-library", git = "https://github.com/LiGoldragon/process-library", rev = "…" }
```

## The standard types

The names a Nexus's generated processes module writes as `process::…`, which
every Nexus shares and a process names, so they live here, in a crate with no
operating system under it, and not in a Nexus: the traits `Processable`, `Giving`,
`Givable`, `Dispatchable`, `Unwindable`, `Ownable` (with `Owner` and the typed
`Giver`), and the types `Caller`
(`ProcessId`, `UserId`), `Answer`, `Unremembered` and `Unobserved`. A Nexus's
processes crate depends on this library under the name `process`.

Each concrete body implements `Processable` and takes `Giving<Self>`. The
generated Process root grants `Givable<ThatProcess>` for each listed gift.
The manager gives the generated aggregate an identity-bound receiver for the
Nexus's shared `Gift`; its dispatch branches wrap the receiver in a concrete
`Giver`. Upgrade from 0.2.0 by repinning the library and ethos-zero together,
regenerating the Process module, and changing the handwritten body bounds to
`Giving<Self>`.

What returns to a process for a thing it gave: nothing for an output; `Answer`
for a record kept; `Result<its answer, Unremembered>` for a change;
`Result<its outcome, Unobserved>` for an emission; and for a process it gave, its
outcome at its end.

## What a close ends

A close after the answer ends nothing. A call's process lives until it yields its
outcome; what it gives its caller after the connection has closed is let go. A
process that declares no outcome, a subscription, ends when its connection
closes: that is its one end. The manager decides none of it: it is the signal
process's, the world the manager is handed, and the Tally Nexus's world shows it,
where a Sum declares its outcome and so a close of its connection shuts the
connection and nothing else.

## The Tally Nexus, its proof of concept

The ethos in `ethos/`: a Signal, a Memory, whose one change, an addition, is
answered by what it came to, and a Process root. It is also the proof of concept
of ethos-zero 32.0.0, pinned in `flake.lock`, which reads the three together:
`tally-process/src/generated/`
is its output for all of them, every path core or alloc, and `tally-process`
builds with no_std unless the `datom` feature asks for the text side. The
Process root's module holds each process's positions (`SumProcess`), what it
takes (`SumTaken`), the enums over the processes and their traits; what is
written by hand is the two bodies, `tally-process/src/lib.rs`.

```sh
ethos-zero 'Generate.{ [ <clone>/ethos/memory.ethos <clone>/ethos/process.ethos <clone>/ethos/signal.ethos ] <clone>/tally-process/src/generated }'
```

High level, what a Nexus author writes, `tally-process/src/lib.rs`: a Sum,
born of a call, gives one Addend process for each number; each Addend, born of
the Sum that gives it, gives one Add to the memory and ends with what it came to;
the Sum takes each Addend's end, and when the last has come gives Summed with the
total and ends. An Addend that overflows ends the Sum, and the Addends still living
are unwound.

Low level, the manager and its world, `tally-nexus/src/lib.rs`: one queue;
the signal process takes each call and shuts each connection a close names; the
memory process keeps the tally and makes one change at a time when nothing
else waits; an end is given to the owner, and the memory lets go of what an ended
process asked. The binary takes the steps as one datom and writes the whole
trace as datom; a step `Settled` waits for the memory to make every change asked of it.

```sh
tally-nexus '[ Called.{ { 100 1000 } Sum.[ 1 2 3 ] } Called.{ { 100 1000 } Sum.[ 4 5 ] } Closed.2 ]'
```

The second sum's connection closes while its addends wait on the memory: nothing
ends, the second sum lives to its outcome and gives `Summed.15` to a connection
that is shut, where it is let go; the first sum answers `Summed.6`.

## Building

The flake gate regenerates Tally and compares the committed modules. It also
compiles the same `give(Add(1))` call with an Addend giver and a Sum giver: only
Addend may give the Memory change, and the Sum case must fail its `Givable`
bound.

```sh
nix run github:Mic92/nix-fast-build -- --remote <builder> --no-nom --no-download \
  --eval-workers 1 --flake 'git+file://<clone>?rev=<commit>#checks.x86_64-linux'
```
