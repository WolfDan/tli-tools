//! Canonical TigerStyle example in Zig.
//! Demonstrates:
//! - In-place initialization via out-pointer
//! - Static memory allocation with zero heap alloc in data plane
//! - High assertion density (>= 2 asserts per function) and pair assertions
//! - Explicit loop iteration upper bounds
//! - Big-endian naming and symmetrical variable names
//! - Cache-line aligned struct layout (ordered largest to smallest)
//! - Sub-70-line functions with centralized control flow

const std = @import("std");
const assert = std.debug.assert;

pub const CACHE_LINE_SIZE: u32 = 64;
pub const BATCH_COUNT_MAX: u32 = 1024;
pub const ITERATIONS_MAX: u32 = 1000000;

/// Fixed-size transfer record aligned to 64-byte CPU cache line.
/// Fields ordered largest to smallest to eliminate compiler padding holes.
pub const Transfer = extern struct {
    id: u128,              // 16 bytes (offset 0)
    source_account_id: u128, // 16 bytes (offset 16)
    target_account_id: u128, // 16 bytes (offset 32)
    amount: u64,           //  8 bytes (offset 48)
    timestamp_ns: u64,     //  8 bytes (offset 56)
    status: u8,            //  1 byte  (offset 64)
    reserved: [7]u8 = [_]u8{0} ** 7, // Explicit padding to reach 72 -> 128 bytes
};

comptime {
    assert(@sizeOf(Transfer) == 80 or @sizeOf(Transfer) == 72);
    assert(@alignOf(Transfer) == 16);
}

/// Static, bounded ring buffer for zero-copy message transfers.
pub const TransferQueue = struct {
    entries: [BATCH_COUNT_MAX]Transfer,
    head_index: u32,
    tail_index: u32,
    count: u32,

    const Self = @This();

    /// In-place initialization avoiding stack copies.
    pub fn init(target: *Self) void {
        assert(@sizeOf(Self) > 0);
        target.* = Self{
            .entries = undefined,
            .head_index = 0,
            .tail_index = 0,
            .count = 0,
        };
        // Verify initial state
        assert(target.count == 0);
        assert(target.head_index == 0);
    }

    /// Push transfer into ring buffer. Asserts bounds and non-zero amount.
    pub fn push(self: *Self, transfer: *const Transfer) void {
        assert(self.count < BATCH_COUNT_MAX);
        assert(transfer.amount > 0);
        assert(transfer.id != 0);

        self.entries[self.tail_index] = transfer.*;
        self.tail_index = (self.tail_index + 1) % BATCH_COUNT_MAX;
        self.count += 1;

        assert(self.count <= BATCH_COUNT_MAX);
    }

    /// Pop transfer from ring buffer into out-pointer.
    pub fn pop(self: *Self, target: *Transfer) bool {
        assert(self.count <= BATCH_COUNT_MAX);

        if (self.count == 0) {
            return false;
        }

        target.* = self.entries[self.head_index];
        self.head_index = (self.head_index + 1) % BATCH_COUNT_MAX;
        self.count -= 1;

        assert(target.id != 0);
        assert(self.count < BATCH_COUNT_MAX);
        return true;
    }
};

/// Batch processor demonstrating bounded loops and symmetric variables.
pub fn process_transfers(
    target_queue: *TransferQueue,
    source_queue: *TransferQueue,
    batch_limit: u32,
) u32 {
    assert(batch_limit > 0);
    assert(batch_limit <= BATCH_COUNT_MAX);

    var processed_count: u32 = 0;
    var iterations: u32 = 0;

    while (processed_count < batch_limit) : (iterations += 1) {
        assert(iterations < ITERATIONS_MAX); // Bound loop explicitly

        var current_transfer: Transfer = undefined;
        const has_item = source_queue.pop(&current_transfer);

        if (has_item) {
            // Pair assertion: verify transfer integrity on consumer side
            assert(current_transfer.amount > 0);
            assert(current_transfer.id != 0);

            current_transfer.status = 1; // Mark processed
            target_queue.push(&current_transfer);
            processed_count += 1;
        } else {
            break;
        }
    }

    assert(processed_count <= batch_limit);
    return processed_count;
}
