use std::collections::HashMap;

use crate::ast::{QualifiedIdentifier, TypeDef, TypeDefId};

pub struct Module {
    type_defs: Vec<TypeDef>,
    type_defs_by_name: HashMap<String, TypeDefId>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            type_defs: Vec::new(),
            type_defs_by_name: HashMap::new(),
        }
    }

    pub fn type_defs(&self) -> impl Iterator<Item = &TypeDef> {
        self.type_defs.iter()
    }

    pub fn from_type_defs(type_defs: Vec<TypeDef>) -> Self {
        let type_defs_by_name = type_defs
            .iter()
            .enumerate()
            .map(|(i, type_def)| (type_def.name.clone(), TypeDefId(i)))
            .collect();

        Self {
            type_defs,
            type_defs_by_name,
        }
    }

    pub fn new_type_def(&mut self, type_def: TypeDef) -> TypeDefId {
        let id = TypeDefId(self.type_defs.len());

        self.type_defs_by_name.insert(type_def.name.clone(), id);
        self.type_defs.push(type_def);

        id
    }

    pub fn type_def(&self, id: TypeDefId) -> &TypeDef {
        &self.type_defs[id.0]
    }

    pub fn type_def_by_name<'a>(&'a self, name: &str) -> Option<&'a TypeDef> {
        Some(self.type_def(*self.type_defs_by_name.get(name)?))
    }
}

struct DatabaseImport {
    lib_suffix: String,
    module: Module,
}

pub struct Database {
    imports: HashMap<String, DatabaseImport>,
    local: Module,
}

impl Database {
    pub fn new(local: Module) -> Self {
        Self {
            imports: HashMap::new(),
            local,
        }
    }

    pub fn local(&self) -> &Module {
        &self.local
    }

    pub fn local_mut(&mut self) -> &mut Module {
        &mut self.local
    }

    pub fn add_module(&mut self, name: String, lib_suffix: impl Into<String>, module: Module) {
        self.imports.insert(
            name,
            DatabaseImport {
                lib_suffix: lib_suffix.into(),
                module,
            },
        );
    }

    pub fn imported_module_mut(&mut self, module_name: &str) -> Option<&mut Module> {
        Some(&mut self.imports.get_mut(module_name)?.module)
    }

    pub fn lookup_module_lib_suffix<'a>(&'a self, module_name: &str) -> Option<&'a str> {
        let import = self.imports.get(module_name)?;
        Some(&import.lib_suffix)
    }

    pub fn lookup_type_def<'a>(&'a self, identifier: &QualifiedIdentifier) -> Option<&'a TypeDef> {
        if let Some(ref module_name) = identifier.module {
            // Importing from another module.
            let import = self.imports.get(module_name)?;
            import.module.type_def_by_name(&identifier.name)
        } else {
            // It's a local definition.
            self.local.type_def_by_name(&identifier.name)
        }
    }
}
