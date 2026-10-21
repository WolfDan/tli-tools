/**
 * Canonical TigerStyle implementation in C99.
 * Demonstrates:
 * - NASA Power of 10 safety guidelines
 * - High assertion density (>= 2 asserts per function)
 * - Zero dynamic heap allocation (static memory only)
 * - Explicit loop iteration bounds
 * - Big-endian naming conventions
 * - Fixed struct packing with symmetrical variable names
 */

#include <stdint.h>
#include <stdbool.h>
#include <assert.h>
#include <string.h>

#define BATCH_COUNT_MAX 512
#define ITERATIONS_MAX 1000000

typedef struct {
    uint64_t account_id;
    uint64_t balance_amount;
    uint32_t status_flags;
    uint16_t currency_code;
    uint8_t  reserved[10]; // Explicit padding for 32-byte alignment
} AccountRecord;

typedef struct {
    AccountRecord entries[BATCH_COUNT_MAX];
    uint32_t count;
    uint32_t head_index;
    uint32_t tail_index;
} AccountQueue;

void account_queue_init(AccountQueue *target) {
    assert(target != NULL);
    assert(sizeof(AccountRecord) == 32);

    memset(target, 0, sizeof(AccountQueue));
    target->count = 0;
    target->head_index = 0;
    target->tail_index = 0;

    assert(target->count == 0);
}

bool account_queue_push(AccountQueue *self, const AccountRecord *source) {
    assert(self != NULL);
    assert(source != NULL);
    assert(source->account_id != 0);

    if (self->count >= BATCH_COUNT_MAX) {
        return false;
    }

    self->entries[self->tail_index] = *source;
    self->tail_index = (self->tail_index + 1) % BATCH_COUNT_MAX;
    self->count += 1;

    assert(self->count <= BATCH_COUNT_MAX);
    return true;
}

bool account_queue_pop(AccountQueue *self, AccountRecord *target) {
    assert(self != NULL);
    assert(target != NULL);
    assert(self->count <= BATCH_COUNT_MAX);

    if (self->count == 0) {
        return false;
    }

    *target = self->entries[self->head_index];
    self->head_index = (self->head_index + 1) % BATCH_COUNT_MAX;
    self->count -= 1;

    assert(target->account_id != 0);
    return true;
}

uint32_t batch_transfer_records(
    AccountQueue *target_queue,
    AccountQueue *source_queue,
    uint32_t batch_limit
) {
    assert(target_queue != NULL);
    assert(source_queue != NULL);
    assert(batch_limit > 0);
    assert(batch_limit <= BATCH_COUNT_MAX);

    uint32_t processed_count = 0;
    uint32_t iterations = 0;

    while (processed_count < batch_limit) {
        iterations += 1;
        assert(iterations < ITERATIONS_MAX); // Finite loop assertion

        AccountRecord record;
        const bool has_record = account_queue_pop(source_queue, &record);
        if (!has_record) {
            break;
        }

        // Pair assertion: verify record state on consumer side
        assert(record.account_id != 0);
        const bool push_success = account_queue_push(target_queue, &record);
        assert(push_success);

        processed_count += 1;
    }

    assert(processed_count <= batch_limit);
    return processed_count;
}
