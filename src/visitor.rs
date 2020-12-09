use ra_ap_hir::{
    self as hir,
    db::HirDatabase,
    Adt,
    BuiltinType,
    Const,
    // Enum,
    EnumVariant,
    Function,
    Module,
    ModuleDef,
    Static,
    // Struct,
    Trait,
    TypeAlias,
    // Union,
};

use ra_ap_ide_db::RootDatabase;

pub use ra_ap_syntax;

pub type Semantics<'db> = ra_ap_hir::Semantics<'db, RootDatabase>;

macro_rules! visiting {
    () => {};
    ($($kind:ident => $method:ident as $type:ident,)*) => {
        trait Visitable: Sized {
            fn accept<T: Visitor>(&self, visitor: &mut T, db: &dyn HirDatabase);
        }

        impl Visitable for ModuleDef {
            fn accept<T: Visitor>(&self, visitor: &mut T, db: &dyn HirDatabase) {
                visitor.pre_visit(self);

                match self {
                    $(ModuleDef::$kind(def) => def.accept(visitor, db),)*
                };

                visitor.post_visit(self);
            }
        }

        pub trait Visitor: Sized {
            /// Call this method to perform a in-order traversal on `node` and its children.
            fn walk(&mut self, module_def: &ModuleDef, db: &dyn HirDatabase) {
                module_def.accept(self, db);
            }

            /// This method is called before visiting a node.
            fn pre_visit(&mut self, _module_def: &ModuleDef) {}

            /// This method is called after visiting a node.
            fn post_visit(&mut self, _module_def: &ModuleDef) {}

            /// This method is called when visiting a node of the given type.
            $(fn $method(&mut self, _def: &hir::$type) {})*
        }
    };
}

visiting!(
    Module => visit_module as Module,
    Function => visit_function as Function,
    Adt => visit_adt as Adt,
    EnumVariant => visit_enum_variant as EnumVariant,
    Const => visit_const as Const,
    Static => visit_static as Static,
    Trait => visit_trait as Trait,
    TypeAlias => visit_type_alias as TypeAlias,
    BuiltinType => visit_builtin_type as BuiltinType,
);

impl Visitable for Module {
    fn accept<T: Visitor>(&self, visitor: &mut T, db: &dyn HirDatabase) {
        visitor.visit_module(self);

        for child in self.clone().declarations(db) {
            child.accept(visitor, db);
        }
    }
}

impl Visitable for Function {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_function(self);
    }
}

impl Visitable for Adt {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_adt(self);
    }
}

impl Visitable for EnumVariant {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_enum_variant(self);
    }
}

impl Visitable for Const {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_const(self);
    }
}

impl Visitable for Static {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_static(self);
    }
}

impl Visitable for Trait {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_trait(self);
    }
}

impl Visitable for TypeAlias {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_type_alias(self);
    }
}

impl Visitable for BuiltinType {
    fn accept<T: Visitor>(&self, visitor: &mut T, _db: &dyn HirDatabase) {
        visitor.visit_builtin_type(self);
    }
}
