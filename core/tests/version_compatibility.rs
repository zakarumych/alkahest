use alkahest_core::{
    Compatibility, CompatibilityError as Error, ElementSchema, Field, Formula, Layout, Primitive,
    Schema, SchemaKind, SizeBound, Variant, element_schema, layout, require_compatibility, schema,
    verify_backward_compatibility, verify_compatibility, verify_forward_compatibility,
};

fn fixed_layout(_: usize) -> Option<Layout> {
    Some(Layout {
        stack: SizeBound::Exact(1),
        heap: SizeBound::Exact(0),
        inhabited: true,
    })
}

fn metadata(kind: SchemaKind) -> Schema {
    Schema {
        version: 0,
        kind,
        layout: fixed_layout,
    }
}

const U8: ElementSchema = ElementSchema {
    schema: schema::<u8, 0>,
    indirect: false,
};
const I8: ElementSchema = ElementSchema {
    schema: schema::<i8, 0>,
    indirect: false,
};
const U16: ElementSchema = ElementSchema {
    schema: schema::<u16, 0>,
    indirect: false,
};

#[test]
fn actual_layout_uses_selected_address_width() {
    use alkahest_core::{Indirect, Str};
    type Record = (Indirect<Str>,);
    let one = layout::<Record, 0>(1).unwrap();
    let four = layout::<Record, 0>(4).unwrap();
    assert_ne!(one.stack, four.stack);
    assert_eq!(one.stack, <Record as Formula>::StackSize::<1>::VALUE);
    assert_eq!(four.stack, <Record as Formula>::StackSize::<4>::VALUE);
    for width in 1..=16 {
        assert!(layout::<Record, 0>(width).is_some());
    }
    assert_eq!(layout::<Record, 0>(0), None);
    assert_eq!(layout::<Record, 0>(17), None);
}

#[test]
fn unsupported_width_is_rejected_even_for_zero_sized_records() {
    for width in [0, 17, usize::MAX] {
        assert_eq!(
            verify_compatibility(schema::<(), 0>(), schema::<(), 0>(), width),
            Err(Error::UnsupportedSizeBytes(width))
        );
    }
}

#[test]
fn equal_sizes_do_not_prove_primitive_compatibility() {
    assert_eq!(
        verify_compatibility(schema::<u8, 0>(), schema::<bool, 0>(), 4),
        Err(Error::EncodingMismatch)
    );
    assert_eq!(
        verify_compatibility(schema::<u32, 0>(), schema::<f32, 0>(), 4),
        Err(Error::EncodingMismatch)
    );
    assert_eq!(
        verify_compatibility(schema::<i8, 0>(), schema::<u8, 0>(), 4),
        Err(Error::EncodingMismatch)
    );
}

#[test]
fn opaque_schema_is_never_proof_even_against_itself() {
    let unknown = metadata(SchemaKind::Unknown);
    assert_eq!(
        verify_compatibility(&unknown, &unknown, 4),
        Err(Error::UnknownSchema)
    );
}

#[test]
fn same_encoding_versions_can_differ() {
    assert_eq!(schema::<u32, 9>().version, 9);
    assert_eq!(
        require_compatibility(
            schema::<u32, 9>(),
            schema::<u32, 0>(),
            Compatibility::Both,
            4
        ),
        Ok(())
    );
}

#[test]
fn nested_child_version_is_pinned_in_static_metadata() {
    let root = metadata(SchemaKind::Record(&[Field {
        name: "nested",
        element: ElementSchema {
            schema: schema::<u32, 7>,
            indirect: false,
        },
    }]));
    let SchemaKind::Record(fields) = root.kind else {
        panic!()
    };
    assert_eq!((fields[0].element.schema)().version, 7);
    assert_eq!((element_schema::<u32, 11>().schema)().version, 11);
}

#[test]
fn record_positions_and_count_are_checked() {
    let a = metadata(SchemaKind::Record(&[
        Field {
            name: "a",
            element: U8,
        },
        Field {
            name: "b",
            element: I8,
        },
    ]));
    let b = metadata(SchemaKind::Record(&[
        Field {
            name: "b",
            element: I8,
        },
        Field {
            name: "a",
            element: U8,
        },
    ]));
    assert_eq!(
        verify_compatibility(&a, &b, 4),
        Err(Error::EncodingMismatch)
    );
    let short = metadata(SchemaKind::Record(&[Field {
        name: "a",
        element: U8,
    }]));
    assert_eq!(
        verify_compatibility(&a, &short, 4),
        Err(Error::FieldCountMismatch)
    );
}

#[test]
fn names_are_diagnostic_not_wire_identity() {
    let a = metadata(SchemaKind::Record(&[Field {
        name: "old_name",
        element: U8,
    }]));
    let b = metadata(SchemaKind::Record(&[Field {
        name: "new_name",
        element: U8,
    }]));
    assert_eq!(verify_compatibility(&a, &b, 4), Ok(()));
}

#[test]
fn nested_layout_and_indirection_are_checked() {
    let direct = metadata(SchemaKind::Record(&[Field {
        name: "x",
        element: U8,
    }]));
    let indirect = metadata(SchemaKind::Record(&[Field {
        name: "x",
        element: ElementSchema {
            indirect: true,
            ..U8
        },
    }]));
    let wider = metadata(SchemaKind::Record(&[Field {
        name: "x",
        element: U16,
    }]));
    assert_eq!(
        verify_compatibility(&direct, &indirect, 4),
        Err(Error::ElementPlacementMismatch)
    );
    assert_eq!(
        verify_compatibility(&direct, &wider, 4),
        Err(Error::LayoutMismatch)
    );
}

#[test]
fn bounded_and_exact_layouts_are_distinct() {
    fn bounded(_: usize) -> Option<Layout> {
        Some(Layout {
            stack: SizeBound::Bounded(1),
            heap: SizeBound::Exact(0),
            inhabited: true,
        })
    }
    let exact = metadata(SchemaKind::Primitive(Primitive::Unsigned(1)));
    let bounded = Schema {
        layout: bounded,
        ..exact
    };
    assert_eq!(
        verify_compatibility(&bounded, &exact, 4),
        Err(Error::LayoutMismatch)
    );
}

#[test]
fn enum_addition_has_direction_and_requirements_enforce_it() {
    let old = metadata(SchemaKind::Enum {
        discriminant_count: 1,
        variants: &[Variant {
            name: "A",
            tag: 0,
            fields: &[],
        }],
    });
    let new = metadata(SchemaKind::Enum {
        discriminant_count: 2,
        variants: &[
            Variant {
                name: "A",
                tag: 0,
                fields: &[],
            },
            Variant {
                name: "B",
                tag: 1,
                fields: &[],
            },
        ],
    });
    assert_eq!(verify_backward_compatibility(&new, &old, 4), Ok(()));
    assert_eq!(
        verify_forward_compatibility(&new, &old, 4),
        Err(Error::MissingVariant(1))
    );
    assert_eq!(
        require_compatibility(&new, &old, Compatibility::Backward, 4),
        Ok(())
    );
    assert_eq!(
        require_compatibility(&new, &old, Compatibility::Forward, 4),
        Err(Error::MissingVariant(1))
    );
    assert_eq!(
        require_compatibility(&new, &old, Compatibility::Both, 4),
        Err(Error::MissingVariant(1))
    );
}

#[test]
fn enum_discriminant_width_change_rejected() {
    let a = metadata(SchemaKind::Enum {
        discriminant_count: 255,
        variants: &[Variant {
            name: "A",
            tag: 0,
            fields: &[],
        }],
    });
    let b = metadata(SchemaKind::Enum {
        discriminant_count: 256,
        variants: &[Variant {
            name: "A",
            tag: 0,
            fields: &[],
        }],
    });
    assert_eq!(
        verify_compatibility(&a, &b, 4),
        Err(Error::EncodingMismatch)
    );
}

#[test]
fn enum_numeric_tag_controls_matching() {
    let a = metadata(SchemaKind::Enum {
        discriminant_count: 2,
        variants: &[Variant {
            name: "same",
            tag: 0,
            fields: &[],
        }],
    });
    let b = metadata(SchemaKind::Enum {
        discriminant_count: 2,
        variants: &[Variant {
            name: "same",
            tag: 1,
            fields: &[],
        }],
    });
    assert_eq!(
        verify_compatibility(&a, &b, 4),
        Err(Error::MissingVariant(1))
    );
}

#[test]
fn duplicate_and_out_of_range_tags_are_invalid() {
    let duplicate = metadata(SchemaKind::Enum {
        discriminant_count: 2,
        variants: &[
            Variant {
                name: "A",
                tag: 0,
                fields: &[],
            },
            Variant {
                name: "B",
                tag: 0,
                fields: &[],
            },
        ],
    });
    let invalid = metadata(SchemaKind::Enum {
        discriminant_count: 1,
        variants: &[Variant {
            name: "A",
            tag: 1,
            fields: &[],
        }],
    });
    assert_eq!(
        verify_compatibility(&duplicate, &duplicate, 4),
        Err(Error::InvalidSchema)
    );
    assert_eq!(
        verify_compatibility(&invalid, &invalid, 4),
        Err(Error::InvalidSchema)
    );
}

#[test]
fn uninhabited_variants_have_no_wire_tag() {
    let a = metadata(SchemaKind::Enum {
        discriminant_count: 1,
        variants: &[
            Variant {
                name: "A",
                tag: 0,
                fields: &[],
            },
            Variant {
                name: "Impossible",
                tag: usize::MAX,
                fields: &[Field {
                    name: "x",
                    element: I8,
                }],
            },
        ],
    });
    let b = metadata(SchemaKind::Enum {
        discriminant_count: 1,
        variants: &[
            Variant {
                name: "A",
                tag: 0,
                fields: &[],
            },
            Variant {
                name: "AnotherImpossible",
                tag: usize::MAX,
                fields: &[Field {
                    name: "x",
                    element: U16,
                }],
            },
        ],
    });
    assert_eq!(
        require_compatibility(&a, &b, Compatibility::Both, 4),
        Ok(())
    );
}

#[test]
fn list_length_domain_is_directional() {
    use alkahest_core::List;
    let old = schema::<List<u8, 2, 4>, 0>();
    let new = schema::<List<u8, 1, 4>, 0>();
    assert_eq!(verify_backward_compatibility(new, old, 4), Ok(()));
    assert_eq!(
        verify_forward_compatibility(new, old, 4),
        Err(Error::LengthRangeMismatch)
    );
}

#[test]
fn fixed_and_variable_list_encodings_are_distinct() {
    let fixed = metadata(SchemaKind::List {
        element: U8,
        min: 3,
        max: 3,
    });
    let variable = metadata(SchemaKind::List {
        element: U8,
        min: 0,
        max: 3,
    });
    assert_eq!(
        verify_compatibility(&variable, &fixed, 4),
        Err(Error::EncodingMismatch)
    );
}

#[test]
fn option_and_record_are_not_interchangeable() {
    let option = metadata(SchemaKind::Option(U8));
    let record = metadata(SchemaKind::Record(&[Field {
        name: "x",
        element: U8,
    }]));
    assert_eq!(
        verify_compatibility(&option, &record, 4),
        Err(Error::EncodingMismatch)
    );
}

#[test]
fn cycles_are_reported_unverified_without_pointer_shortcuts() {
    fn recursive() -> &'static Schema {
        &RECURSIVE
    }
    static RECURSIVE: Schema = Schema {
        version: 0,
        layout: fixed_layout,
        kind: SchemaKind::Record(&[Field {
            name: "next",
            element: ElementSchema {
                schema: recursive,
                indirect: true,
            },
        }]),
    };
    assert_eq!(
        verify_compatibility(&RECURSIVE, &RECURSIVE, 4),
        Err(Error::RecursionLimit)
    );
}

use alkahest_core::SizeType;
