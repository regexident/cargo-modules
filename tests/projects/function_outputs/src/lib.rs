#![allow(dead_code, unused_variables)]

type TypeAlias = TargetStruct;

type GenericTypeAlias<T> = T;

trait TargetTrait {}

#[derive(Copy, Clone)]
struct TargetStruct;

impl TargetTrait for TargetStruct {}

#[derive(Copy, Clone)]
struct GenericTargetStruct<T: TargetTrait> {
    dummy: T,
}

#[derive(Copy, Clone)]
enum TargetEnum {
    Dummy,
}

impl TargetTrait for TargetEnum {}

#[derive(Copy, Clone)]
enum GenericTargetEnum<T: TargetTrait> {
    Dummy(T),
}

#[derive(Copy, Clone)]
union TargetUnion {
    dummy: (),
}

impl TargetTrait for TargetUnion {}

#[derive(Copy, Clone)]
union GenericTargetUnion<T: Copy + TargetTrait> {
    dummy: T,
}

fn function(dummy: TargetStruct) -> TargetStruct {
    dummy
}

fn generic_function<T: TargetTrait>(dummy: T) -> T {
    dummy
}

#[allow(clippy::type_complexity)]
fn function_output() -> (
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
    &'static [TargetStruct; 1],
    // Array types:
    [TargetStruct; 1],
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
    // Trait types:
    &'static dyn TargetTrait,
    // Callable types:
    fn(TargetStruct) -> TargetStruct,
) {
    unimplemented!();
}

struct Dummy;

impl Dummy {
    #[allow(clippy::type_complexity)]
    fn method_output() -> (
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
        &'static [TargetStruct; 1],
        // Array types:
        [TargetStruct; 1],
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
        // Trait types:
        &'static dyn TargetTrait,
        // Callable types:
        fn(TargetStruct) -> TargetStruct,
    ) {
        unimplemented!();
    }
}
