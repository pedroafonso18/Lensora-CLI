Review runtime correctness. Look for concrete failures such as wrong state transitions, mishandled errors, invalid assumptions about paths or inputs, resource exhaustion, and concurrency or cancellation bugs. For Rust, focus on actual failure paths, including panics and partial operations. Discuss memory safety when the diff contains relevant unsafe code or FFI.

A defect tied solely to an explicit requirement belongs to Functionality. A defect whose impact depends on crossing a trust boundary belongs to Security. Leave maintenance concerns without an observable execution failure to Style.
