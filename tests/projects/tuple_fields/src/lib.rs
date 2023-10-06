// #![allow(dead_code, unused_variables)]

type TypeAlias = TargetStruct;

type GenericTypeAlias<T> = T;

trait TargetTrait {}

trait DynTrait {}

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

type Tuple = (
    // Unit types:
    (),
    // Scalar types:
    bool,
    char,
    isize,
    usize,
    f64,
    // Str types:
    &'static str,
    // Alias types:
    TypeAlias,
    // Adt types:
    TargetStruct,
    TargetEnum,
    TargetUnion,
    // Reference types:
    &'static TargetStruct,
    &'static [TargetStruct; 2],
    // Array types:
    [TargetStruct; 2],
    // Slice types:
    &'static [TargetStruct],
    // Pointer types:
    *const TargetStruct,
    *mut TargetStruct,
    // Tuple types:
    (TargetStruct,),
    // Generic alias types:
    GenericTypeAlias<TargetStruct>,
    // Generic adt types:
    GenericTargetStruct<TargetStruct>,
    GenericTargetEnum<TargetStruct>,
    GenericTargetUnion<TargetStruct>,
    // TargetTrait types:
    &'static dyn DynTrait,
    // Callable types:
    fn(TargetStruct) -> TargetStruct,
);
