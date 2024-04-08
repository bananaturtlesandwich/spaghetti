use unreal_asset::{
    exports::*,
    properties::Property,
    types::{PackageIndex, PackageIndexTrait},
    Asset, Import,
};

mod fprop;
mod kismet;
mod uprop;

pub fn transplant<C: std::io::Seek + std::io::Read, D: std::io::Seek + std::io::Read>(
    index: usize,
    recipient: &mut Asset<C>,
    donor: &Asset<D>,
) {
    let mut children = get_actor_exports(index, donor, recipient.asset_data.exports.len());
    // resolve all import references from exports
    let import_offset = recipient.imports.len() as i32;
    let mut imports = Vec::new();
    for child in &mut children {
        on_import_refs(child, |index| {
            if let Some(import) = donor.get_import(*index) {
                index.index = match recipient.find_import_no_index(
                    &import.class_package,
                    &import.class_name,
                    &import.object_name,
                ) {
                    // sometimes e.g for GEN_VARIABLEs you want those imports
                    Some(existing)
                        if donor.get_import(import.outer_index).is_some_and(|imp| {
                            recipient
                                .get_import(PackageIndex::new(existing))
                                .is_some_and(|import| {
                                    imp.class_package.eq_content(&import.class_package)
                                        && imp.class_name.eq_content(&import.class_name)
                                        && imp.object_name.eq_content(&import.object_name)
                                })
                        }) =>
                    {
                        existing
                    }
                    _ => {
                        -import_offset
                            - match imports.iter().position(|imp: &Import| {
                                imp.class_package.eq_content(&import.class_package)
                                    && imp.class_name.eq_content(&import.class_name)
                                    && imp.object_name.eq_content(&import.object_name)
                            }) {
                                Some(existing) => existing + 1,
                                None => {
                                    imports.push(import.clone());
                                    // this actually pads perfectly so no need for + 1
                                    imports.len()
                                }
                            } as i32
                    }
                }
            }
        })
    }
    // finally add the exports
    recipient.asset_data.exports.append(&mut children);

    // resolve all import references from exports
    let mut i = 0;
    // use a while loop because the vector is expanding while the operation occurs & imports.len() updates every loop
    while i < imports.len() {
        if let Some(parent) = donor.get_import(imports[i].outer_index) {
            imports[i].outer_index.index = match recipient.find_import_no_index(
                &parent.class_package,
                &parent.class_name,
                &parent.object_name,
            ) {
                Some(existing)
                    if donor.get_import(parent.outer_index).is_some_and(|imp| {
                        recipient
                            .get_import(PackageIndex::new(existing))
                            .is_some_and(|import| {
                                imp.class_package.eq_content(&import.class_package)
                                    && imp.class_name.eq_content(&import.class_name)
                                    && imp.object_name.eq_content(&import.object_name)
                            })
                    }) =>
                {
                    existing
                }
                _ => {
                    -import_offset
                        - match imports.iter().position(|import: &Import| {
                            import.class_package.eq_content(&parent.class_package)
                                && import.class_name.eq_content(&parent.class_name)
                                && import.object_name.eq_content(&parent.object_name)
                        }) {
                            Some(existing) => existing + 1,
                            None => {
                                imports.push(parent.clone());
                                // this actually pads perfectly so no need for + 1
                                imports.len()
                            }
                        } as i32
                }
            }
        }
        i += 1;
    }
    recipient.imports.append(&mut imports);
}

/// gets all exports related to the given actor
fn get_actor_exports<C: std::io::Seek + std::io::Read>(
    index: usize,
    asset: &Asset<C>,
    offset: usize,
) -> Vec<Export<PackageIndex>> {
    // get references to all the actor's children
    let mut child_indexes: Vec<PackageIndex> = asset.asset_data.exports[index]
        .get_base_export()
        .create_before_serialization_dependencies
        .iter()
        .filter(|dep| dep.is_export())
        // dw PackageIndex is just a wrapper around i32 which is cloned by default anyway
        .cloned()
        .collect();
    // add the top-level actor reference
    if !child_indexes
        .iter()
        .any(|i| (i.index - 1) as usize == index)
    {
        child_indexes.insert(0, PackageIndex::new(index as i32 + 1))
    }

    // get all the exports from those indexes
    let mut children: Vec<_> = child_indexes
        .iter()
        .filter_map(|index| asset.get_export(*index))
        // i'm pretty sure i have to clone here so i can modify then insert data
        .cloned()
        .collect();

    let package_offset = (offset + 1) as i32;
    // update export references to what they will be once added
    for (i, child_index) in child_indexes.into_iter().enumerate() {
        for child in children.iter_mut() {
            on_export_refs(child, |index| {
                if index == &child_index {
                    index.index = package_offset + i as i32;
                }
            });
        }
    }
    children
}

/// on all possible export references
fn on_export_refs(export: &mut Export<PackageIndex>, mut func: impl FnMut(&mut PackageIndex)) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in &mut norm.properties {
            on_props(prop, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    export
        .create_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .create_before_serialization_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .serialization_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    func(&mut export.outer_index);
}

fn on_norm(norm: &mut NormalExport<PackageIndex>, func: &mut impl FnMut(&mut PackageIndex)) {
    for prop in &mut norm.properties {
        on_props(prop, func);
    }
}

fn on_struct(struc: &mut StructExport<PackageIndex>, mut func: &mut impl FnMut(&mut PackageIndex)) {
    if let Some(next) = struc.field.next.as_mut() {
        func(next)
    }
    func(&mut struc.super_struct);
    struc.children.iter_mut().for_each(&mut func);
    for prop in &mut struc.loaded_properties {
        fprop::on_fprop(prop, &mut func)
    }
    if let Some(script) = struc.script_bytecode.as_mut() {
        for inst in script {
            kismet::on_kismet(inst, &mut func)
        }
    }
    on_norm(&mut struc.normal_export, func)
}

/// on all of an export's possible references to imports
fn on_import_refs(export: &mut Export<PackageIndex>, mut func: impl FnMut(&mut PackageIndex)) {
    match export {
        Export::BaseExport(_) => (),
        Export::ClassExport(class) => {
            class.func_map.values_mut().for_each(&mut func);
            func(&mut class.class_within);
            for interface in &mut class.interfaces {
                func(&mut interface.class)
            }
            func(&mut class.class_generated_by);
            func(&mut class.class_default_object);
            on_struct(&mut class.struct_export, &mut func);
        }
        Export::EnumExport(en) => on_norm(&mut en.normal_export, &mut func),
        Export::LevelExport(lev) => {
            lev.actors.iter_mut().for_each(&mut func);
            func(&mut lev.model);
            lev.model_components.iter_mut().for_each(&mut func);
            func(&mut lev.level_script);
            func(&mut lev.nav_list_start);
            func(&mut lev.nav_list_end);
            on_norm(&mut lev.normal_export, &mut func)
        }
        Export::NormalExport(norm) => on_norm(norm, &mut func),
        Export::PropertyExport(prop) => {
            uprop::on_uprop(&mut prop.property, &mut func);
            on_norm(&mut prop.normal_export, &mut func);
        }
        Export::RawExport(_) => (),
        Export::StringTableExport(str) => on_norm(&mut str.normal_export, &mut func),
        Export::StructExport(struc) => on_struct(struc, &mut func),
        Export::UserDefinedStructExport(uds) => {
            for prop in &mut uds.default_struct_instance {
                on_props(prop, &mut func)
            }
            on_struct(&mut uds.struct_export, &mut func)
        }
        Export::FunctionExport(fun) => on_struct(&mut fun.struct_export, &mut func),
        Export::DataTableExport(data) => {
            for data in &mut data.table.data {
                for entry in &mut data.value {
                    on_props(entry, &mut func);
                }
            }
            on_norm(&mut data.normal_export, &mut func);
        }
        Export::WorldExport(world) => {
            func(&mut world.persistent_level);
            world.extra_objects.iter_mut().for_each(&mut func);
            world.streaming_levels.iter_mut().for_each(&mut func);
            on_norm(&mut world.normal_export, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    func(&mut export.class_index);
    func(&mut export.template_index);
    // not serialization_before_serialization because only the first few map exports have those
    export
        .serialization_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .create_before_serialization_dependencies
        .iter_mut()
        .for_each(&mut func);
}

/// on any possible references stashed away in properties
fn on_props(prop: &mut Property, func: &mut impl FnMut(&mut PackageIndex)) {
    match prop {
        Property::ObjectProperty(obj) => {
            func(&mut obj.value);
        }
        Property::ArrayProperty(arr) => {
            for entry in &mut arr.value {
                on_props(entry, func);
            }
        }
        Property::MapProperty(map) => {
            for val in map.value.values_mut() {
                on_props(val, func);
            }
        }
        Property::SetProperty(set) => {
            for entry in &mut set.value.value {
                on_props(entry, func);
            }
            for entry in &mut set.removed_items.value {
                on_props(entry, func);
            }
        }
        Property::DelegateProperty(del) => func(&mut del.value.object),
        Property::MulticastDelegateProperty(del) => {
            for delegate in &mut del.value {
                func(&mut delegate.object)
            }
        }
        Property::MulticastSparseDelegateProperty(del) => {
            for delegate in &mut del.value {
                func(&mut delegate.object)
            }
        }
        Property::MulticastInlineDelegateProperty(del) => {
            for delegate in &mut del.value {
                func(&mut delegate.object)
            }
        }
        Property::StructProperty(struc) => {
            for entry in &mut struc.value {
                on_props(entry, func);
            }
        }
        _ => (),
    }
}
