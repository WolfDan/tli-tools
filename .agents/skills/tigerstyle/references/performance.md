# TigerStyle Performance Guide

> "The lack of back-of-the-envelope sketches is the root of all evil. Think about performance from the outset. The time to solve performance, and get the 1000x wins, is in the design phase, when you can't profile." — [TigerStyle](https://tigerstyle.dev/)

Performance in TigerStyle is an upfront architectural concern. By understanding the physical characteristics of hardware—mechanical sympathy—we design software that works with the grain of modern CPUs, memory hierarchies, storage devices, and networks.

---

## 1. The Four Primary Colors

Every system can be modeled using 4 primary resources and 2 textures:

```
┌──────────────────────────────────────────────────────────────┐
│  PRIMARY COLORS       │  TEXTURES                            │
├───────────────────────┼──────────────────────────────────────┤
│  1. Network           │  • Bandwidth (throughput, bytes/sec) │
│  2. Storage (Disk)    │  • Latency   (round-trip time, ns)   │
│  3. Memory (RAM)      │                                      │
│  4. Compute (CPU)     │                                      │
└──────────────────────────────────────────────────────────────┘
```

### Hierarchy of Optimization
Optimize for the slowest resource first:
$$\text{Network} \longrightarrow \text{Storage/Disk} \longrightarrow \text{Memory} \longrightarrow \text{Compute/CPU}$$

> [!TIP]
> A memory cache miss (100 ns) multiplied millions of times in a tight loop quickly exceeds the latency of a single SSD I/O operation. Always factor in **frequency of access**.

---

## 2. Control Plane vs. Data Plane Separation

A crisp separation between the Control Plane and the Data Plane unlocks both maximum safety and maximum performance:

### The Control Plane
* Orchestrates state machines, routing, coordination, admission control, and configuration.
* Executes infrequently (e.g. per batch, per connection, or per epoch).
* Employs heavy assertions, safety checks, and complex branching.

### The Data Plane
* Processes streams of items directly in memory.
* Executes at maximum frequency on hot paths.
* Zero dynamic allocations, zero serialization/deserialization, minimal branching.
* Designed for CPU sprinting—letting the CPU run sequential vector instructions on contiguous memory blocks without context switches.

---

## 3. Zero-Copy & Zero-Deserialization

In modern hardware, **per-core memory bandwidth is a major bottleneck**. Copying buffers or parsing variable-length messages wastes memory bandwidth and trashes CPU caches.

### 3.1 Direct Wire-to-Memory Alignment
* Use fixed-size, statically structured binary headers and records.
* Avoid protobuf, JSON, or custom serialization on the data plane.
* Cast byte slices directly to typed pointers after validating magic numbers, checksums, and version tags.

### 3.2 Cache-Line Alignment (64 Bytes)
* Align high-frequency structs to 64 bytes (`@alignOf(T) == 64`) to ensure a single struct fits neatly across CPU cache lines without splitting across cache line boundaries.
* Order struct fields from **largest to smallest** (`u128` → `u64` → `u32` → `u16` → `u8`) to eliminate compiler-inserted padding holes:

```zig
pub const Account = extern struct {
    id: u128,              // 16 bytes (offset 0)
    user_id: u128,         // 16 bytes (offset 16)
    balance: u64,          //  8 bytes (offset 32)
    timestamp_ns: u64,     //  8 bytes (offset 40)
    flags: u32,            //  4 bytes (offset 48)
    code: u16,             //  2 bytes (offset 52)
    status: u8,            //  1 byte  (offset 54)
    reserved: [9]u8 = [_]u8{0} ** 9, // Explicitly zeroed padding to reach 64 bytes
};

comptime {
    assert(@sizeOf(Account) == 64);
    assert(@alignOf(Account) == 16);
}
```

---

## 4. In-Place Initialization

Avoid constructing large structs on the stack and returning them by value, which causes unnecessary memory copies and risk of stack overflow:

```zig
// ✅ TigerStyle: In-place initialization via out-pointer
pub fn init(target: *StateTable, capacity: u32) !void {
    assert(capacity > 0);
    target.* = StateTable{
        .capacity = capacity,
        .count = 0,
        .entries = undefined,
    };
}

// ❌ Anti-pattern: Intermediate stack allocation and copy
pub fn init(capacity: u32) !StateTable {
    return StateTable{ ... };
}
```

---

## 5. Batching & Amortization

* **Make the Roads Bigger**: Replace single-item operations with batched arrays.
* Process arrays of records (`[]const Message`) in a single syscall or disk transfer.
* Amortize fixed overheads (syscalls, fsyncs, network round-trips, lock acquisitions) across hundreds or thousands of operations.

---

## 6. Preventing Buffer Bleeds

A **buffer bleed** occurs when uninitialized or stale memory in padding or unused slice capacities leaks into disk blocks or network packets.

* Always explicitly zero padding bytes.
* Zero out buffers upon release or initialization.
* Maintain strict determinism: identical inputs must yield bit-for-bit identical binary representations on disk and wire.
