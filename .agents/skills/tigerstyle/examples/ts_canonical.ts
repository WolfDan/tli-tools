/**
 * Canonical TigerStyle implementation in TypeScript.
 * Demonstrates:
 * - Pre-allocated fixed-size typed arrays for static memory behavior
 * - Explicit runtime assertion contracts (>= 2 asserts per function)
 * - Explicit loop iteration bounds
 * - Big-endian naming conventions
 * - Symmetrical parameter names
 */

function assert(condition: boolean, message: string = "Assertion failed"): asserts condition {
    if (!condition) {
        throw new Error(`[TigerStyle Assertion Violation] ${message}`);
    }
}

export const BATCH_COUNT_MAX = 512;
export const ITERATIONS_MAX = 1_000_000;

export interface LogEntry {
    readonly log_id: bigint;
    readonly timestamp_ms: number;
    readonly severity_code: number;
}

export class BoundedLogQueue {
    private readonly entries: Array<LogEntry | null>;
    private head_index: number = 0;
    private tail_index: number = 0;
    private entry_count: number = 0;

    constructor(private readonly capacity_limit: number = BATCH_COUNT_MAX) {
        assert(capacity_limit > 0, "Capacity must be positive");
        assert(capacity_limit <= BATCH_COUNT_MAX, "Capacity must not exceed BATCH_COUNT_MAX");

        this.entries = new Array(capacity_limit).fill(null);
        assert(this.entries.length === capacity_limit, "Entries array initialized");
    }

    public push(source_entry: LogEntry): boolean {
        assert(source_entry.log_id > 0n, "Log ID must be non-zero");
        assert(this.entry_count <= this.capacity_limit, "Count cannot exceed capacity");

        if (this.entry_count >= this.capacity_limit) {
            return false;
        }

        this.entries[this.tail_index] = source_entry;
        this.tail_index = (this.tail_index + 1) % this.capacity_limit;
        this.entry_count += 1;

        assert(this.entry_count <= this.capacity_limit, "Postcondition: valid count");
        return true;
    }

    public pop(): LogEntry | null {
        assert(this.entry_count <= this.capacity_limit, "Invariant check");

        if (this.entry_count === 0) {
            return null;
        }

        const entry = this.entries[this.head_index];
        this.entries[this.head_index] = null; // Clear reference
        this.head_index = (this.head_index + 1) % this.capacity_limit;
        this.entry_count -= 1;

        assert(entry !== null, "Popped item must not be null");
        assert(entry.log_id > 0n, "Popped item must have valid ID");
        return entry;
    }

    public count(): number {
        assert(this.entry_count >= 0, "Count non-negative");
        assert(this.entry_count <= this.capacity_limit, "Count within capacity");
        return this.entry_count;
    }
}

export function transfer_log_batch(
    target_queue: BoundedLogQueue,
    source_queue: BoundedLogQueue,
    batch_limit: number
): number {
    assert(batch_limit > 0, "Batch limit must be positive");
    assert(batch_limit <= BATCH_COUNT_MAX, "Batch limit within maximum bounds");

    let processed_count = 0;
    let iterations = 0;

    while (processed_count < batch_limit) {
        iterations += 1;
        assert(iterations < ITERATIONS_MAX, "Loop iterations within finite bounds");

        const entry = source_queue.pop();
        if (!entry) {
            break;
        }

        // Pair assertion: verify entry in consumer
        assert(entry.log_id > 0n, "Entry ID valid");
        const push_ok = target_queue.push(entry);
        assert(push_ok, "Target queue must accept entry within batch limits");

        processed_count += 1;
    }

    assert(processed_count <= batch_limit, "Processed count satisfies batch limit");
    return processed_count;
}
