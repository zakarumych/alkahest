use alkahest_core::{Formula, Indirect, List, MakeIter, Serialize, advanced::write_usize, small};

#[test]
fn trivial_wide_size_fields_clear_high_bytes() {
    struct SizeField;
    impl Formula for SizeField {
        type StackSize<const SIZE_BYTES: usize> = alkahest_core::ExactSize<SIZE_BYTES>;
        type HeapSize<const SIZE_BYTES: usize> = alkahest_core::ExactSize<0>;
        const INHABITED: bool = true;
    }
    impl Serialize<SizeField> for usize {
        fn serialize<S: alkahest_core::Serializer>(
            &self,
            mut serializer: S,
        ) -> Result<(), S::Error> {
            serializer.write_usize(*self)
        }
    }
    let mut bytes = [0xa5; 16];
    let len = alkahest_core::manual_size::serialize::<SizeField, _, 16>(&7usize, &mut bytes)
        .expect("serialize wide size field");
    assert_eq!(len, 16);
    assert_eq!(bytes, 7u128.to_le_bytes());
}

fn check_buffers<F, T>(value: &T)
where
    F: Formula,
    T: Serialize<F>,
{
    let required = small::serialized_size::<F, _>(value);
    let mut expected = vec![0; required];
    assert_eq!(small::serialize::<F, _>(value, &mut expected), Ok(required));

    for capacity in 0..=required + 8 {
        let mut actual = vec![0xa5; capacity];
        match small::serialize_or_size::<F, _>(value, &mut actual) {
            Ok(written) => {
                assert!(capacity >= required);
                assert_eq!(written, required);
                assert_eq!(&actual[..written], expected);
            }
            Err(error) => {
                assert!(capacity < required);
                assert_eq!(error.required, required);
            }
        }
    }

    let required = small::pack_size::<F, _>(value);
    let mut expected = vec![0; required];
    assert_eq!(small::pack::<F, _>(value, &mut expected), Ok(required));

    for capacity in 0..=required + 8 {
        let mut actual = vec![0xa5; capacity];
        match small::pack_or_size::<F, _>(value, &mut actual) {
            Ok(written) => {
                assert!(capacity >= required);
                assert_eq!(written, required);
                assert_eq!(&actual[..written], expected);
            }
            Err(error) => {
                assert!(capacity < required);
                assert_eq!(error.required, required);
            }
        }
    }
}

#[test]
fn size_reporting_buffers_match_checked_buffers() {
    check_buffers::<u32, _>(&0x12345678u32);
    check_buffers::<(u32, u32), _>(&(11u32, 22u32));
    check_buffers::<List<Indirect<u32>>, _>(&vec![11u32, 22, 33]);
    check_buffers::<((Indirect<u32>, u8), u16), _>(&((11u32, 22u8), 33u16));
    check_buffers::<(), _>(&());
}

#[test]
fn size_reporting_buffers_handle_unknown_sizes_after_exhaustion() {
    let values = MakeIter(|| (0u32..12).filter(|value| value % 2 == 0));
    assert!(<MakeIter<_> as Serialize<List<u32>>>::size_hint::<1>(&values).is_none());
    check_buffers::<List<u32>, _>(&values);
}

fn check_size_boundary<const WIDTH: usize>() {
    let max = (1usize << (WIDTH * 8)) - 1;
    let mut bytes = [0; WIDTH];
    write_usize::<_, WIDTH>(max, 0, bytes.as_mut_slice());
    assert_eq!(bytes, [0xff; WIDTH]);
    assert!(
        std::panic::catch_unwind(|| {
            let mut bytes = [0; WIDTH];
            write_usize::<_, WIDTH>(max + 1, 0, bytes.as_mut_slice());
        })
        .is_err()
    );
}

#[test]
fn size_fields_reject_first_unrepresentable_value() {
    check_size_boundary::<1>();
    check_size_boundary::<2>();
    check_size_boundary::<3>();
    #[cfg(target_pointer_width = "64")]
    check_size_boundary::<4>();
}

#[test]
fn full_width_size_fields_accept_usize_max() {
    let mut native = [0; size_of::<usize>()];
    write_usize::<_, { size_of::<usize>() }>(usize::MAX, 0, native.as_mut_slice());
    assert_eq!(native, usize::MAX.to_le_bytes());

    let mut wide = [0; 16];
    write_usize::<_, 16>(usize::MAX, 0, wide.as_mut_slice());
    assert_eq!(&wide[..native.len()], native);
    assert!(wide[native.len()..].iter().all(|byte| *byte == 0));
}

#[test]
fn small_lists_accept_255_elements_and_reject_256() {
    let mut bytes = [0; 1];
    assert_eq!(
        small::serialize::<List<()>, _>(&vec![(); 255], &mut bytes),
        Ok(1)
    );
    assert_eq!(bytes, [255]);
    assert_eq!(
        small::deserialize::<List<()>, Vec<()>>(&bytes)
            .expect("valid list")
            .len(),
        255
    );
    assert!(
        std::panic::catch_unwind(|| {
            let mut bytes = [0; 1];
            let _ = small::serialize::<List<()>, _>(&vec![(); 256], &mut bytes);
        })
        .is_err()
    );
}
