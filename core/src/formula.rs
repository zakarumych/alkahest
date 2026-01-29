pub trait Formula<const SIZE_BYTES: u8>: 'static {
    /// Maximum size of stack this formula occupies.
    ///
    /// If `None`, the size is unbounded.
    const MAX_STACK_SIZE: Option<usize>;

    /// Specifies that MAX_STACK_SIZE is exact.
    const EXACT_SIZE: bool;

    /// Signals that heap is not used for serialzation.
    ///
    /// Heap is used for indirect serialization.
    const HEAPLESS: bool;
}
