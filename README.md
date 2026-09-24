# freelance-escrow

A Solana program (Anchor) implementing a two-party escrow between a client and a freelancer, with deadline enforcement, a client review window, and independent recovery paths for both sides if the other goes silent.

Built as a learning/portfolio project to develop real intuition for Solana's account model, PDAs, CPI vs. direct lamport authority, and Anchor's constraint system — not copied from a tutorial.

## Why an escrow needs to be on-chain

The trust problem: a client wants proof the freelancer gets paid *only* on delivery, and the freelancer wants proof the client can't just walk away after the work is done. Neither party should have to trust the other, or a third-party platform, to hold the money fairly. Putting the funds in a Program Derived Address (PDA) — an account with no private key, controlled entirely by program logic — means *neither* party can unilaterally move the funds. Only the code paths below can, and only under the conditions they enforce.

## Instruction flow

```
initialize_escrow          (client, signer)
    ↓  status: Pending
fund_escrow                (client, signer)
    ↓  status: Funded
accept_job                 (freelancer, signer)
    ↓  status: Accepted
complete_job                (freelancer, signer)
    ↓  status: Completed
release_funds               (client, signer)  →  status: Released
    or
claim_after_deadline        (freelancer, signer, time-gated)  →  status: Released
    or, from Funded/Accepted:
cancel_and_refund           (client, signer, time-gated)  →  status: Cancelled
```

## Instructions

### `initialize_escrow`
Client creates the escrow PDA and stores the job terms: freelancer's pubkey (as data, not yet an authority), `amount`, `deadline`, `grace_period`, `review_window`, `job_id`. Freelancer isn't required to sign here — the relationship is proposed by the client, agreed to later when the freelancer calls `accept_job`.

### `fund_escrow`
Client transfers `amount` (read from the escrow account itself, never re-supplied by the caller) from their own wallet into the escrow PDA via CPI into the System Program — required because the client's wallet is owned by System Program, not by this program.

### `accept_job`
Freelancer signs, proving they agree to the terms already on-chain. Gated by `has_one = freelancer` (only the freelancer named at init can accept) and `status == Funded` (can't accept unfunded work — added after catching that gap mid-build) and `current_time < deadline` (accepting after the deadline has already passed is meaningless).

### `complete_job`
Freelancer marks delivery. Records `completed_at` via the `Clock` sysvar — never caller-supplied, so a freelancer can't backdate a late submission. Gated by `status == Accepted` and `current_time <= deadline + grace_period` (the grace period exists specifically to avoid a hard cutoff on lateness).

### `release_funds`
Client actively approves and funds are paid to the freelancer. Since the escrow PDA is *owned by this program*, the transfer is done by directly manipulating the account's lamports field — no CPI needed, because the program already has authority over its own account. (CPI/`invoke_signed` is only required to move funds out of accounts owned by *other* programs, such as a plain wallet.)

### `claim_after_deadline`
Freelancer's fallback if the client goes silent after completion. Requires `current_time >= completed_at + review_window + grace_period`. `review_window` exists specifically to block a timing attack where a freelancer submits at the last possible moment hoping the client won't notice in time.

### `cancel_and_refund`
Client's fallback if the freelancer never delivers. Valid from `Funded` or `Accepted` (but not `Completed` — a client can't cancel out from under a freelancer who already delivered). Requires `current_time >= deadline`.

## PDA design

```
seeds = [b"escrow", client.key().as_ref(), job_id.to_le_bytes().as_ref()]
```

- `job_id` is scoped **per-client**, not globally unique — deliberate. The escrow's identity only needs to be unique within one client's own jobs; global uniqueness was considered and explicitly deferred (see below).
- Namespacing with `b"escrow"` future-proofs against seed collisions if other PDA types are added to this program later.

## Security model

Every fund-moving or state-mutating instruction is gated by two independent layers, enforced *before* the handler body runs:

- **Who** — `has_one` constraints verify the signer matches the authority already stored on the account (not just "an account was supplied," but "this specific party is provably authorized").
- **When / what state** — `constraint =` and `require!` checks on `status` and on-chain time (`Clock` sysvar) prevent out-of-order calls and premature/late execution.

No instruction trusts caller-supplied data for anything the program can determine itself (`status`, `completed_at`, transfer `amount` are always read from the account, never re-passed as arguments).

## Testing

Full instruction coverage in `tests/`, one file per instruction, sharing setup helpers via `tests/common/mod.rs`. Uses LiteSVM (in-process Solana simulator) rather than a live validator. Time-gated instructions (`accept_job`, `complete_job`, `claim_after_deadline`, `cancel_and_refund`) are tested in both directions — proving the instruction correctly *rejects* a too-early call and correctly *succeeds* once the simulated clock is advanced past the required threshold.

```bash
anchor build
anchor test
```

## V1 scope decisions

- No dispute/rejection instruction — if a client is unhappy with a completed submission and doesn't act within `review_window + grace_period`, the freelancer can claim. This is a "silence = approval" design; disputes are treated as a V2 concern.
- No freelancer-reassignment instruction — if the original freelancer never delivers, the client's recovery path is `cancel_and_refund` followed by initializing a new escrow.

## V2 ideas (not built)

- Dispute/concern resolution mechanism, with an additional funding pool if resolving a concern exceeds the original job scope
- Direct freelancer reassignment on an existing (still-funded) escrow, without a full refund/reinitialize cycle