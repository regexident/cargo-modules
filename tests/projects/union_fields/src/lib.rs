#![allow(dead_code, unused_variables)]

type TypeAlias = TargetStruct;

type GenericTypeAlias<T> = T;

trait TargetTrait {}

#[derive(Copy, Clone)]
struct TargetStruct;

impl TargetTrait for TargetStruct {}

#[derive(Copy, Clone)]
struct GenericTargetStruct<T: Copy + TargetTrait> {
    t: T,
}

#[derive(Copy, Clone)]
enum TargetEnum {}

impl TargetTrait for TargetEnum {}

#[derive(Copy, Clone)]
enum GenericTargetEnum<T: Copy + TargetTrait> {
    T(T),
}

#[derive(Copy, Clone)]
union TargetUnion {
    dummy: TargetStruct,
}

impl TargetTrait for TargetUnion {}

#[derive(Copy, Clone)]
union GenericTargetUnion<T: Copy + TargetTrait> {
    t: T,
}

pub union Union {
    // Unit types:
    unit_field: (),
    // Scalar types:
    bool_field: bool,
    char_field: char,
    int_field: isize,
    uint_field: usize,
    float_field: f64,
    // Str types:
    str_field: &'static str,
    // Alias types:
    type_alias_field: TypeAlias,
    // Adt types:
    struct_field: TargetStruct,
    enum_field: TargetEnum,
    union_field: TargetUnion,
    // Reference types:
    reference_field: &'static TargetStruct,
    array_reference_field: &'static [TargetStruct; 2],
    // Array types:
    array_field: [TargetStruct; 2],
    // Slice types:
    slice_field: &'static [TargetStruct],
    // Pointer types:
    const_ptr_field: *const TargetStruct,
    mut_ptr_field: *mut TargetStruct,
    // Tuple types:
    tuple_field: (TargetStruct,),
    // Generic alias types:
    generic_type_alias_field: GenericTypeAlias<TargetStruct>,
    // Generic adt types:
    generic_struct_field: GenericTargetStruct<TargetStruct>,
    generic_enum_field: GenericTargetEnum<TargetStruct>,
    generic_union_field: GenericTargetUnion<TargetStruct>,
    // Trait types:
    dyn_trait_field: &'static dyn TargetTrait,
    // Callable types:
    callable_field: fn(TargetStruct) -> TargetStruct,
}
