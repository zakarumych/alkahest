use alkahest_core::{
    Indirect, List, MakeIter, Serialize,
    advanced::{Buffer, VecBuffer},
    large,
};

#[test]
fn reservations_use_existing_capacity_without_repeated_stack_moves() {
    let mut bytes = Vec::with_capacity(128);
    let capacity = bytes.capacity();
    let pointer = bytes.as_ptr();
    for stack in 0..capacity {
        let mut buffer = VecBuffer::new(&mut bytes);
        let reserved = buffer.reserve(0, stack, 1).expect("infallible buffer");
        assert_eq!(reserved.len(), capacity);
        assert_eq!(reserved.as_ptr(), pointer);
        assert!(reserved[capacity - stack..].iter().all(|byte| *byte == 7));
        buffer.write_stack(stack, &[7]);
    }
    assert_eq!(bytes, vec![7; capacity]);
}

#[test]
fn growing_reservation_preserves_heap_and_stack() {
    let mut bytes = vec![1, 2, 3, 4];
    bytes.resize(bytes.capacity(), 0);
    let old_len = bytes.len();
    bytes[old_len - 2..].copy_from_slice(&[8, 9]);
    let mut buffer = VecBuffer::new(&mut bytes);
    let reserved = buffer
        .reserve_heap(2, 2, old_len)
        .expect("infallible buffer");
    assert_eq!(reserved.len(), old_len + 2);
    assert_eq!(&reserved[..2], &[1, 2]);
    assert_eq!(bytes.len(), bytes.capacity());
    assert!(bytes.len() >= old_len + 4);
    assert_eq!(&bytes[bytes.len() - 2..], &[8, 9]);
}

#[test]
fn unknown_length_lists_roundtrip_with_growing_and_reused_buffers() {
    let expected: Vec<u32> = (0..1000).collect();
    let values = MakeIter(|| expected.iter().copied().filter(|_| true));
    assert!(<MakeIter<_> as Serialize<List<u32>>>::size_hint::<4>(&values).is_none());
    for mut bytes in [Vec::new(), Vec::with_capacity(16384), vec![0xa5; 32]] {
        for _ in 0..2 {
            let len = large::serialize_to_vec::<List<u32>, _>(&values, &mut bytes);
            assert_eq!(
                large::deserialize::<List<u32>, Vec<u32>>(&bytes[..len]).expect("flat list"),
                expected
            );
            let len = large::serialize_to_vec::<List<Indirect<u32>>, _>(&values, &mut bytes);
            assert_eq!(
                large::deserialize::<List<Indirect<u32>>, Vec<u32>>(&bytes[..len])
                    .expect("indirect list"),
                expected
            );
        }
    }
}
