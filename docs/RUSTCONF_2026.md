---
marp: true
theme: RUSTCONF
paginate: true
title: Life Before Main
description: Scatter at link time, gather before main
---

# <!-- fit --> There's Life Before Main In Rust

## Matt Mastracci - Sept. 9 2026

<table>

<tr><td>

Adapted from:
https://grack.com/blog/2026/06/11/life-before-main/

 - https://bsky.app/profile/mmastrac.bsky.social
 - https://x.com/mmastrac
 - https://github.com/mmastrac

</td><td>

🦀

</td><td>

<style scoped>
table, tr, td { border: 0 !important; }
pre { font-size: 13px; line-height: 13px; white-space: pre; background: transparent; border: 0; }
</style>

<pre>
█▀▀▀▀▀█  ▀██▄ ▀▄█ █▀██▄▀▀ █▀▀▀▀▀█    
█ ███ █ █▀▀▄▀ ▀▄  ▄█▀ ▀██ █ ███ █    
█ ▀▀▀ █ █▀▄█▄▀▀▀▀▄ ▄▀ ▄██ █ ▀▀▀ █    
▀▀▀▀▀▀▀ █▄▀▄█ █ █ ▀▄█▄▀ ▀ ▀▀▀▀▀▀▀    
▀ ▀▀▀▀▀▄ ▄█ ▄▄ ▄ ███▄ ▀█▀ ██▀██ ▄    
██▀▄▀▀▀▄▄▀██▀█▀█▄█▄ ▄▀█▄▄▀▀ █▄█ ▄    
 ▀█ ▄█▀▀████▀ ▀█▀ ▄▀ ▄  ▀▀ ███ ▄▄    
▀▀ ██▄▀ ▄█  ▄▄▄█▀ █  ▄▄▄▀██  ▀▀▀     
▀▀▄▄▄█▀▄▄▄█ █▄█▀▄▀▄▄▄ ▀█▀▀▄█▀▄█ ▀    
█▀▄▀▄▄▀  ▄▄▄▀▄▄▄▄ ▀▄ █▀ ▄▀▀██▄█ ▀    
▄ █▀█ ▀▄▄██ █  █▄ ▀▄▄  ▄▀ ████  ▄    
█  ▀▀▀▀  ▀▀ ▄▄▀ ▄█ ▀█▄█▀ █▄▀▄▀█▀▄    
▀ ▀▀  ▀▀█▄▀█ ▄ ▄▄▄▀▄▄ ▀▀█▀▀▀█ ▄█▄    
█▀▀▀▀▀█ ▄▄▀ █ ▀█ █▄▄█████ ▀ █▄█ ▀    
█ ███ █ ███ █ ▄█▀ ▄▀▄   ▀█▀▀██ ▄     
█ ▀▀▀ █ ▀  ▄█▄ █▀ ▀▀███ ███▀▄▄▀      
▀▀▀▀▀▀▀ ▀▀  ▀ ▀▀  ▀▀   ▀   ▀  ▀▀     
</pre>

</td></tr>
</table>

---

# Not Just Theory

The crates we'll be discussing here are running on 100,000+ systems
every day.

---

# Something Concrete

Real-world use cases:

 - Distributed registration
 - String tables
 - C library initialization

---

# Where We All Start (One Crate)

```
   +-----------------------------------+
   |  my_app                           |
   |                                   |
   |  types   a   b   c   collector    |
   |      ^   ^   ^   ^   ^            |
   |      +---+---+---+---+            |
   +-----------------------------------+
```
```rust
let mut commands: Vec<&dyn Command> = vec![];
commands.push(&HelpCommand::default());
commands.push(&VersionCommand::default());
commands.push(&ListCommand::default());
commands.push(&AddCommand::default());
```

---

# The Collector Knows Everyone

```
                 +---------------+
                 |  shared types |
                 +---------------+
                         ^
         +---------------+---------------+
         |               |               |
     +-------+       +-------+       +-------+
     | mod_a |       | mod_b |       | mod_c |
     +-------+       +-------+       +-------+
         ^               ^               ^
         +---------------+---------------+
                         |
                  +-------------+
                  |  collector  |
                  +-------------+
```

---

# ... And Has To Know Everything

```
                 +---------------+
                 |  shared types |
                 +---------------+
                         ^
         +---------------+---------------+-------+
         |               |               |       |
     +-------+       +-------+       +-------+ +-------+
     | mod_a |       | mod_b |       | mod_c | | mod_d |
     +-------+       +-------+       +-------+ +-------+
         ^               ^               ^       ^
         +---------------+---------------+-------+
                         |               new edge, every time
                  +-------------+
                  |  collector  |
                  +-------------+
```

---

# Where We'd Like To Be

```
                 +--------------------+
                 |  shared types      |
                 |  collector         |  <- declares + gathers
                 +--------------------+
                    ^     ^     ^    ^
         +----------+     |     |    +--------+
         |                |     |             |
     +-------+       +-------+  |         +--------+
     | mod_a |       | mod_b |  |         |  user  |
     +-------+       +-------+  |         +--------+
                               +-------+
                               | mod_c |
                               +-------+

      scatter        scatter    scatter      read

```

---

# Prior Art

`linkme`: https://crates.io/crates/linkme

 - Supports all major desktop platforms
 - Makes a read-only slice
 - Does not support WASM

`inventory`: https://crates.io/crates/inventory

 - Supports everything
 - Requires one `#[ctor]` per item, iteration only

---

# Before Main Is Real

<style scoped>
section { padding: 40px 60px; }
h1 { font-size: 1.2em; margin: 0 0 0.15em; }
pre { font-size: 16px; line-height: 1.3; margin: 0 0 0.4em; padding: 0.2em 0.6em; }
table pre { font-size: 18px; line-height: 1.25; }
</style>

warning: pseudo-assembler ahead

<table>

<tr><td>

```asm
; kernel jumps here! (ELF e_entry)
_start:                         
        ; ... <set up the stack>

        ; never returns
        call    __libc_start_main   

__libc_start_main:
        call    __libc_setup_tls   ; thread-local storage
        call    __malloc_init      ; the heap exists now
        call    __init_stdio       ; stdin/stdout/stderr

        call    __libc_csu_init    ; <-- walk .init_array
        
        call    main               ; finally!
        call    __libc_csu_fini    ; clean up
        call    exit               ; sayonara!
```

</td><td>

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;

</td><td>

```asm
@section .init_array
    <init-fn-ptr-1>
    <init-fn-ptr-2>
    <init-fn-ptr-3>
    ...

@section .fini_array
    <fini-fn-ptr-1>
    <fini-fn-ptr-2>
    <fini-fn-ptr-3>
    ...

@section .code
    main:
    init_fn_1:
    ...
@section .data
    ...
```

</td></tr>

</table>

---

# Nothing Calls These

`#[ctor]` marks a function that lands in `.init_array`.

<table>

<tr><td>

```rust
use ctor::ctor;

#[ctor(unsafe, priority = 101)]
fn init1() {
    println!("Initializing (first)!");
}

#[ctor(unsafe, priority = 201)]
fn init2() {
    println!("Initializing (second)!");
}

fn main() {
    println!("Main!")
}
```

</td><td>

```
$ cargo run
Initializing (first)!
Initializing (second)!
Main!
```

</td></tr>

</table>


---

# Everyone Registers Themselves

```rust
#[ctor(unsafe, priority = 101)]
fn register_help() {
    GLOBAL_COMMANDS::register(&HelpCommand::default());
}

#[ctor(unsafe, priority = 201)]
fn register_version() {
    GLOBAL_COMMANDS::register(&VersionCommand::default());
}

#[ctor(unsafe, priority = 301)]
fn register_list() {
    GLOBAL_COMMANDS::register(&ListCommand::default());
}
```

---

# The Linker "Kind of" Knows This

Using `ctor` is _Better_ but not _perfect_:

 - Allocation before main
 - Heap resize churning (unknown up-front count)
 - No easy ordering guarantees

<hr>

BUT... Every one of these symbols is available to the linker. If only we could organize them.

---

# Name Your Section

```rust
#[used]
#[unsafe(link_section = "our_commands")]
static HELP: Command = Command::new("help", "Show help", run_help);
#[used]
#[unsafe(link_section = "our_commands")]
static LIST: Command = Command::new("list", "List items", run_list);
```

---

# Where Does It End?

```rust
unsafe extern "C" {
    #[link_name = "__start_our_commands"]
    static START: MaybeUninit<()>;
               // ^^^^^^^^^^^   We'll come back to this! 
    #[link_name = "__stop_our_commands"]
    static STOP: MaybeUninit<()>;
}
```

---

# These Aren't Pointers

These are _placement_ symbols, not pointers!

```rust
fn commands() -> &'static [Command] {
    unsafe {
        let start = &raw const START as *const Command;
        let stop  = &raw const STOP  as *const Command;
        std::slice::from_raw_parts(start, stop.offset_from(start) as usize)
    }
}
```

The pseudo-assembly we generate:

```asm
@section our_commands:
  __start_our_commands:  ; <-- zero-sized symbol!
  HELP:
    ; <HELP command data>
  LIST:
    ; <LIST command data>
  __stop_our_commands:
```

---

# Enter `link-section`

```rust
// `registration` crate
use link_section::{section, in_section};

#[section(typed)]
static COMMANDS: link_section::TypedSection<Command>;


// These can live anywhere!
use registration::COMMANDS;

#[in_section(COMMANDS)]
static HELP_COMMAND: Command = HelpCommand;

#[in_section(COMMANDS)]
static VERSION_COMMAND: Command = VersionCommand::new("1.0");

#[in_section(COMMANDS)]
static LIST_COMMAND: Command = ListCommand::new();
```

---

# Scatter (Easy), Gather (Linear)

We can pick up items from anywhere, but there's no indexing or order.

---

# Nothing Else Is Running

Life before main is:

 - single-threaded
 - _happens-before_ `main`

---

# Sort Before, Search After

```rust
#[section(mutable)]
static COMMANDS: link_section::TypedMutableSection<Command>;
                               //   ^^^^^^^ (!)

#[in_section(COMMANDS)]
const HELP_COMMAND: Command = HelpCommand;
// ^-- n.b.!

#[ctor(unsafe, priority = 101)]
fn sort_commands() {
    COMMANDS.sort_unstable();
}

// O(log(N)) rather than O(N)
fn lookup(name: &str) -> Option<Command> {
    COMMANDS.binary_search_by_key(&name, |c| c.name()).ok().map(|i| COMMANDS[i])
}
```

---

# UnsafeCell Or Segfault

```rust
// This is a nightly-only experimental API
pub unsafe auto trait Freeze { }
```

`UnsafeCell` strips the `Freeze` trait.

```llvm
; rodata
; static FOO: Command = ...;
@FOO = internal constant %Command { ... }, section "our_commands"

; data
; static BAR: UnsafeCell<Command> = ...;
@BAR = internal global   %Command { ... }, section "our_commands"
```

---

# Enter `scattered-collect`

Superset of `linkme` and `inventory`, replaces `ctor`/`link_section` manual
registration.

Supports slices (sorted/unsorted), maps/sets, filesystem (coming soon!)

```rust
use scattered_collect::*;

#[scatter]
static COMMANDS: ScatteredMap;

#[gather]
static COMMANDS: (&'static str, Command) = ("help", HelpCommand);
#[gather]
static COMMANDS: (&'static str, Command) = ("version", VersionCommand::new("1.0"));
#[gather]
static COMMANDS: (&'static str, Command) = ("list", ListCommand::new());
```

---
# ScatteredMap Internals

Allocates space for items _and_ metadata at link time.

Fixup function called at boot.

Uses an optimized, read-only version of `HashMap`'s Swiss-table layout.

<style scoped>
pre {
    font-size: 14px;
}
</style>

<table>

<tr><td>

`metadata` section

```
00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
```

</td><td>

`#[ctor]` →

</td><td>

`metadata` section

```
E3 E1 12 34 01 02 43 11  3E C8 B0 C3 12 99 36 01
3E C8 B0 C3 12 99 36 01  E3 E1 12 34 01 02 43 11
```

</td></tr>

<tr><td colspan="3">

`data` section

```
hash = 0x0011223344556677   data = <struct record 1>
hash = 0x9988776655443322   data = <struct record 2>
```

</td></tr>

</table>

---

# 99% Of The Time It Works 100% Of The Time

 - #479 illegal access errors on windows

```
[ELIFECYCLE] Command failed with exit code 3221225477.
```

---

# Declaring The Pointer

Start/stop markers on Windows are zero-valued statics.

Provenance taints pointers derived from these known-zero bytes.

Therefore LLVM will happily zero the whole slice.

Not enough time in this presentation to cover it, but:

 - `std::hint::black_box` alone is not sufficient
 - `expose_provenance` and `with_exposed_provenance` + `black_box`: sufficient escape hatch.
 - `with_addr` on an `asm!`-defined symbol is more pedantically correct.
 - Rust has no clear way to say "the linker made this array"

---

# When Not To

 - dead-code elimination
 - `ctor`s can't panic
 - action-at-a-distance

---

# WASM, Briefly

WASM doesn't support link sections today, so they are emulated by
`link-section`.

---

# Scatter Link-Time, Gather Pre-Main

```
     mod_a      mod_b      mod_c               consumer
       |          |          |                    |
       +----------+----------+--------------------+
                             |
                    +--------------------+
                    |  shared types      |
                    |  #[section] VALUES |
                    +--------------------+

     scatter ....................... link time
     gather ........................ before main
     read .......................... after main, lock-free
```


---

# <!-- fit --> There's Life Before Main In Rust

<style scoped>
table, tr, td { border: 0 !important; }
pre { font-size: 13px; line-height: 13px; white-space: pre; background: transparent; border: 0; }
</style>

<table>

<tr><td>

https://grack.com/blog/2026/06/11/life-before-main/

`linktime` crates:
 - https://crates.io/crates/ctor
 - https://crates.io/crates/dtor
 - https://crates.io/crates/link-section
 - https://crates.io/crates/scattered-collect

</td><td>

<pre>
█▀▀▀▀▀█  ▀██▄ ▀▄█ █▀██▄▀▀ █▀▀▀▀▀█    
█ ███ █ █▀▀▄▀ ▀▄  ▄█▀ ▀██ █ ███ █    
█ ▀▀▀ █ █▀▄█▄▀▀▀▀▄ ▄▀ ▄██ █ ▀▀▀ █    
▀▀▀▀▀▀▀ █▄▀▄█ █ █ ▀▄█▄▀ ▀ ▀▀▀▀▀▀▀    
▀ ▀▀▀▀▀▄ ▄█ ▄▄ ▄ ███▄ ▀█▀ ██▀██ ▄    
██▀▄▀▀▀▄▄▀██▀█▀█▄█▄ ▄▀█▄▄▀▀ █▄█ ▄    
 ▀█ ▄█▀▀████▀ ▀█▀ ▄▀ ▄  ▀▀ ███ ▄▄    
▀▀ ██▄▀ ▄█  ▄▄▄█▀ █  ▄▄▄▀██  ▀▀▀     
▀▀▄▄▄█▀▄▄▄█ █▄█▀▄▀▄▄▄ ▀█▀▀▄█▀▄█ ▀    
█▀▄▀▄▄▀  ▄▄▄▀▄▄▄▄ ▀▄ █▀ ▄▀▀██▄█ ▀    
▄ █▀█ ▀▄▄██ █  █▄ ▀▄▄  ▄▀ ████  ▄    
█  ▀▀▀▀  ▀▀ ▄▄▀ ▄█ ▀█▄█▀ █▄▀▄▀█▀▄    
▀ ▀▀  ▀▀█▄▀█ ▄ ▄▄▄▀▄▄ ▀▀█▀▀▀█ ▄█▄    
█▀▀▀▀▀█ ▄▄▀ █ ▀█ █▄▄█████ ▀ █▄█ ▀    
█ ███ █ ███ █ ▄█▀ ▄▀▄   ▀█▀▀██ ▄     
█ ▀▀▀ █ ▀  ▄█▄ █▀ ▀▀███ ███▀▄▄▀      
▀▀▀▀▀▀▀ ▀▀  ▀ ▀▀  ▀▀   ▀   ▀  ▀▀     
</pre>

</td></tr>

</table>
