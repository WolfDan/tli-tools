# TigerStyle Safety Guide

> "Correctness is necessary but not sufficient for safety. To be safe, a program must not only run correctly. It must apply defense-in-depth and verify itself while running, to run correctly or else shut down if it detects that it has violated expectations." — [TigerStyle](https://tigerstyle.dev/)

Safety is the highest priority in TigerStyle. Every design decision and line of code must prioritize predictability, fault containment, and invariant verification over convenience.

---

## 1. NASA's Power of 10 for Safety-Critical Code

TigerStyle adopts the principles from Gerard J. Holzmann's *Power of 10: Rules for Developing Safety-Critical Code*:

1. **Avoid Complex Flow Constructs**: No `goto`, `setjmp`, `longjmp`, or direct/indirect recursion.
2. **Fixed Upper Bounds on All Loops**: Every loop must have an explicit, statically provable or runtime-asserted termination bound.
3. **Static Memory Allocation**: Do not use dynamic memory allocation (`malloc`, `free`, `realloc`) after initialization.
4. **Short Functions**: Hard limit of 70 lines per function.
5. **High Assertion Density**: Average at least 2 assertions per function.
6. **Narrowest Scope**: Declare variables at the smallest possible scope.
7. **Strict Return Value Checks**: Never ignore the return value of non-void functions.
8. **Limited Preprocessor Use**: Avoid macro obfuscation.
9. **Pointer Discipline**: Restrict pointer dereferencing and eliminate pointer arithmetic ambiguity.
10. **Zero Compiler Warnings**: Compile from day one with the strictest compiler flags enabled.

---

## 2. Relentless Assertions: The First Line of Defense

Assertions detect programmer errors, convert catastrophic silent corruption into immediate liveness issues (fail-fast), and serve as machine-checkable documentation.

### 2.1 Assertion Density Rule
* **Every function must average >= 2 assertions.**
* Check:
  1. Arguments and preconditions at the function entry.
  2. Return values and postconditions before exiting.
  3. Loop invariants and internal state transitions.

```zig
pub fn ring_buffer_push(self: *RingBuffer, item: Item) void {
    assert(self.count < self.capacity); // Precondition
    assert(item.id != 0);               // Argument validity

    self.items[self.write_index] = item;
    self.write_index = (self.write_index + 1) % self.capacity;
    self.count += 1;

    assert(self.count <= self.capacity); // Postcondition
}
```

### 2.2 Pair Assertions
For every critical invariant or property, assert it across at least two distinct code paths or lifecycle boundaries:
* Before writing to disk AND immediately after reading from disk.
* Before transmitting over the wire AND immediately after deserializing at the receiver.
* In the producer before enqueue AND in the consumer after dequeue.

### 2.3 Positive Space AND Negative Space
* **Positive Space**: Assert what *must* be true.
* **Negative Space**: Assert what *must not* happen (forbidden state transitions, invalid bounds, unaligned offsets).

### 2.4 Compile-Time Assertions (`comptime`)
Verify struct sizes, alignment constraints, and constant relationships at compile time:
```zig
comptime {
    assert(@sizeOf(Header) == 64);
    assert(@alignOf(Header) == 64);
    assert(MESSAGE_SIZE_MAX % SECTOR_SIZE == 0);
    assert(BATCH_SIZE_MAX <= QUEUE_CAPACITY);
}
```

### 2.5 Split Compound Assertions & Implication
* Prefer multiple individual assertions over combined logical operators:
  ```zig
  // ✅ Excellent
  assert(index >= 0);
  assert(index < count);

  // ❌ Avoid
  assert(index >= 0 and index < count);
  ```
* Use single-line conditional assertions for implications:
  ```zig
  if (is_leader) assert(lease_timeout_ns > current_time_ns);
  ```

---

## 3. Explicit Bounds & Finite Execution

Unbounded loops, unbounded queues, and unbounded retries lead to tail latency spikes, memory exhaustion, and deadlocks.

1. **Loop Iteration Limits**:
   ```zig
   var iterations: u32 = 0;
   while (condition) : (iterations += 1) {
       assert(iterations < ITERATIONS_MAX);
       // loop body
   }
   ```
2. **Explicitly-Sized Types**: Always use fixed-width types (`u8`, `u16`, `u32`, `u64`, `u128`). Avoid architecture-dependent types like `usize` in persisted data and protocol definitions.
3. **Queue Capacities**: Every queue, channel, and ring buffer must have a fixed, pre-allocated capacity constant.

---

## 4. Static Memory Allocation

Dynamic allocation introduces memory fragmentation, unpredictable latency spikes, allocation failures under load, and use-after-free bugs.

* **Startup Initialization**: Allocate all buffers, pools, and data structures at program startup during the initialization phase.
* **Zero Runtime Allocation**: No allocation or freeing during steady-state request processing (the data plane).
* **Stack Discipline**: Avoid deep call stacks or large stack allocations; pass out-pointers to initialize data in-place.

---

## 5. Scope Minimization & Semantic Gaps

* **Declare Variables Where Used**: Do not declare variables at the top of a function if they are only used 30 lines later.
* **Eliminate POCPOU**: Prevent *Place-of-Check to Place-of-Use* bugs by reducing the time and distance between verification and utilization.
* **Immutability by Default**: Mark variables `const` unless mutation is strictly necessary.

---

## 6. Centralized Control Flow & Pure Leaves

* **Push `if`s Up, Push `for`s Down**:
  - Coordinator functions handle branching, state transitions, and coordination.
  - Leaf functions perform arithmetic, transformation, or verification without complex nested branches.
* **70-Line Function Limit**: If a function exceeds 70 lines, decompose it into a parent coordinator and pure worker helpers.
* **Positive Condition Formulation**:
  ```zig
  // ✅ Clean, positive invariant
  if (index < count) {
      // Invariant holds
  } else {
      // Invariant violated
  }

  // ❌ Harder to parse negation
  if (index >= count) {
      // Fallback
  }
  ```

---

## 7. 100% Error Handling

Production analyses show that over 90% of catastrophic distributed systems outages stem from improper handling of explicitly signaled errors.

* Never ignore errors (`try`, `catch`, or explicit matching required).
* Test error paths exhaustively, including fault injection for I/O and network failures.
* Distinguish between **Operating Errors** (expected environmental conditions, handled gracefully) and **Programmer Errors** (unexpected invariant breaches, caught by assertions that crash immediately).
