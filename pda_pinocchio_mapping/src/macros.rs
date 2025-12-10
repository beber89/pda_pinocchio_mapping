/**
 * Constructs a [`Mapping`] using the caller crate’s program ID.
 *
 * This macro retrieves `crate::ID` from the invoking program and passes it to
 * [`Mapping::new`], ensuring that the mapping is always associated with the
 * program that uses this library rather than the library crate itself.
 *
 * Behavior:
 * - At expansion time, verifies that `crate::ID` exists and has type `Pubkey`.
 * - Produces a fully initialized [`Mapping`] using:
 *     - the caller’s program ID (`crate::ID`),
 *     - the provided `name`,
 *     - the provided `payer`.
 *
 * Usage:
 * ```
 * let m = mapping!(b"my_mapping", payer_account);
 * ```
 *
 * Requirements:
 * - The caller crate must expose a public `ID: Pubkey` constant.
 * - `name` must be a byte-slice identifier.
 * - `payer` must be an `AccountInfo` reference.
 */
#[macro_export]
macro_rules! mapping {
    ($name:expr, $payer:expr) => {{
        // Fail early if ID doesn't exist or is the wrong type
        let program_id: &pinocchio::pubkey::Pubkey = &crate::ID;

        $crate::Mapping::new(program_id, $name, $payer)
    }};
}
