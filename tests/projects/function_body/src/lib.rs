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

fn function_body() {
    // Unit types:
    #[allow(clippy::let_unit_value)]
    let unit_binding: () = ();
    // Scalar types:
    let bool_binding: bool = false;
    let char_binding: char = 'a';
    let int_binding: isize = -42;
    let uint_binding: usize = 42;
    let float_binding: f64 = 42.0;
    // Str types:
    let str_binding: &'static str = "hello world";
    // Alias types:
    let type_alias_binding: TypeAlias = TargetStruct;
    // Adt types:
    let struct_binding: TargetStruct = TargetStruct;
    let enum_binding: TargetEnum = TargetEnum::Dummy;
    let union_binding: TargetUnion = TargetUnion { dummy: () };
    // Reference types:
    let reference_binding: &'static TargetStruct = &TargetStruct;
    let array_reference_binding: &'static [TargetStruct; 1] = &[TargetStruct];
    // Array types:
    let array_binding: [TargetStruct; 1] = [TargetStruct];
    // Slice types:
    let slice_binding: &'static [TargetStruct] = &[TargetStruct];
    // Pointer types:
    let const_ptr_binding: *const TargetStruct = &TargetStruct;
    let mut_ptr_binding: *mut TargetStruct = &mut TargetStruct;
    // Tuple types:
    let tuple_binding: (TargetStruct,) = (TargetStruct,);
    // Generic alias types:
    let generic_type_alias_binding: GenericTypeAlias<TargetStruct> = TargetStruct;
    // Generic adt types:
    let generic_struct_binding: GenericTargetStruct<TargetStruct> = GenericTargetStruct {
        dummy: TargetStruct,
    };
    let generic_enum_binding: GenericTargetEnum<TargetStruct> =
        GenericTargetEnum::Dummy(TargetStruct);
    let generic_union_binding: GenericTargetUnion<TargetStruct> = GenericTargetUnion {
        dummy: TargetStruct,
    };
    // Trait types:
    let dyn_trait_binding: &'static dyn TargetTrait = &TargetStruct;
    // Callable types:
    let function_binding: fn(TargetStruct) -> TargetStruct = function;
    let generic_function_binding: fn(TargetStruct) -> TargetStruct = generic_function;
    let closure_binding: fn(TargetStruct) -> TargetStruct = |dummy| dummy;
}

struct Dummy;

impl Dummy {
    fn method_body() {
        // Unit types:
        #[allow(clippy::let_unit_value)]
        let unit_binding: () = ();
        // Scalar types:
        let bool_binding: bool = false;
        let char_binding: char = 'a';
        let int_binding: isize = -42;
        let uint_binding: usize = 42;
        let float_binding: f64 = 42.0;
        // Str types:
        let str_binding: &'static str = "hello world";
        // Alias types:
        let type_alias_binding: TypeAlias = TargetStruct;
        // Adt types:
        let struct_binding: TargetStruct = TargetStruct;
        let enum_binding: TargetEnum = TargetEnum::Dummy;
        let union_binding: TargetUnion = TargetUnion { dummy: () };
        // Reference types:
        let reference_binding: &'static TargetStruct = &TargetStruct;
        let array_reference_binding: &'static [TargetStruct; 1] = &[TargetStruct];
        // Array types:
        let array_binding: [TargetStruct; 1] = [TargetStruct];
        // Slice types:
        let slice_binding: &'static [TargetStruct] = &[TargetStruct];
        // Pointer types:
        let const_ptr_binding: *const TargetStruct = &TargetStruct;
        let mut_ptr_binding: *mut TargetStruct = &mut TargetStruct;
        // Tuple types:
        let tuple_binding: (TargetStruct,) = (TargetStruct,);
        // Generic alias types:
        let generic_type_alias_binding: GenericTypeAlias<TargetStruct> = TargetStruct;
        // Generic adt types:
        let generic_struct_binding: GenericTargetStruct<TargetStruct> = GenericTargetStruct {
            dummy: TargetStruct,
        };
        let generic_enum_binding: GenericTargetEnum<TargetStruct> =
            GenericTargetEnum::Dummy(TargetStruct);
        let generic_union_binding: GenericTargetUnion<TargetStruct> = GenericTargetUnion {
            dummy: TargetStruct,
        };
        // Trait types:
        let dyn_trait_binding: &'static dyn TargetTrait = &TargetStruct;
        // Callable types:
        let function_binding: fn(TargetStruct) -> TargetStruct = function;
        let generic_function_binding: fn(TargetStruct) -> TargetStruct = generic_function;
        let closure_binding: fn(TargetStruct) -> TargetStruct = |dummy| dummy;
    }
}
