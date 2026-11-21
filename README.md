# process-library

The process library of Process.4: a process is an object at rest in the
manager's table, given one thing at a time, and it gives. Nothing in it waits;
what waits, the queue, the sockets and the disk, is the world the manager is
handed. It builds with no operating system under it unless the `datom` feature
asks for the text side.

Lifted from `~/Code/claude-harness-nexus/process` at f0b425a, with two
additions: `Identity::owned`, the identity a process bears a process under,
and `Identity::owner`, read back from it.

```toml
process = { package = "process-library", git = "https://github.com/LiGoldragon/process-library", rev = "…" }
```

## The Tally Nexus, its proof of concept

The ethos in `ethos/`: a Signal, a Memory, the Library holding the change
and its answer until the Memory page places them, and a Process root.
ethos-zero checks the first three; it knows no Process root yet, so
`tally-process/src/generated.rs` is written by hand as the generator would
write it.

High level, what a Nexus author writes, `tally-process/src/lib.rs`: a Sum,
born of a call, gives one Addend process for each number; each Addend gives
one Add to the memory and ends with the Addition it takes back; the Sum takes
each Addend's end, and when the last has come gives Summed with the total. An
Addend that overflows ends the Sum, and the Addends still living are unwound.

Low level, the manager and its world, `tally-nexus/src/lib.rs`: one queue;
the signal process takes each call and closes each connection; the memory
process keeps the tally and makes one change at a time when nothing else
waits; an end is given to the owner, and the memory lets go of what an ended
process asked. The binary takes the steps as one datom and writes the whole
trace as datom.

```sh
tally-nexus '[ Called.Sum.[ 1 2 3 ] Called.Sum.[ 4 5 ] Closed.2 ]'
```

The second sum's connection closes while its addends wait on the memory: its
two Addends end Unwound, the last born first, then the Sum, and the memory
lets go of their two Adds; the first sum answers `Summed.6`.

## Building

```sh
nix run github:Mic92/nix-fast-build -- --remote <builder> --no-nom --no-download \
  --eval-workers 1 --flake 'git+file://<clone>?rev=<commit>#checks.x86_64-linux'
```
