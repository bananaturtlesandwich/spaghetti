use unreal_asset::{exports::*, properties::Property, types::PackageIndex, Asset, Import};

mod fprop;
mod kismet;
mod uprop;

pub fn transplant<C: std::io::Seek + std::io::Read, D: std::io::Seek + std::io::Read>(
    index: usize,
    recipient: &mut Asset<C>,
    donor: &Asset<D>,
    class: i32,
) {
    let mut child = get_export(index, donor, recipient.asset_data.exports.len(), class);
    let base = child.get_base_export_mut();
    base.object_name = base
        .object_name
        .get_content(|name| recipient.add_fname(name.trim_start_matches("hook_")));
    // resolve all import references from exports
    let import_offset = recipient.imports.len() as i32;
    let mut imports = Vec::new();
    on_import_refs(&mut child, |index| {
        if let Some(import) = donor.get_import(*index) {
            index.index = match recipient.find_import_no_index(
                &import.class_package,
                &import.class_name,
                &import.object_name,
            ) {
                Some(existing) => existing,
                None => {
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
    });
    // finally add the exports
    recipient.asset_data.exports.push(child);

    // resolve all import references from imports
    let mut i = 0;
    // use a while loop because the vector is expanding while the operation occurs (imports.len() updates every loop)
    while i < imports.len() {
        if let Some(parent) = donor.get_import(imports[i].outer_index) {
            imports[i].outer_index.index = match recipient.find_import_no_index(
                &parent.class_package,
                &parent.class_name,
                &parent.object_name,
            ) {
                Some(existing) => existing,
                None => {
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

/// gets export redirected to the recipient
fn get_export<C: std::io::Seek + std::io::Read>(
    index: usize,
    asset: &Asset<C>,
    offset: usize,
    class: i32,
) -> Export<PackageIndex> {
    let mut child = asset.asset_data.exports[index].clone();

    let package_offset = (offset + 1) as i32;
    // update export references to what they will be once added
    let old = child.get_base_export_mut().outer_index.index;
    on_export_refs(&mut child, |i| {
        if i.index == index as i32 + 1 {
            i.index = package_offset as i32;
        } else if i.index == old {
            i.index = class;
        }
    });
    let base = child.get_base_export_mut();
    if let Some(i) = base
        .create_before_create_dependencies
        .iter_mut()
        .find(|i| i.index == old)
    {
        i.index = class;
    }
    base.outer_index.index = class;
    child
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

fn on_export(export: &mut Export<PackageIndex>, mut func: &mut impl FnMut(&mut PackageIndex)) {
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
            on_struct(&mut class.struct_export, func);
        }
        Export::EnumExport(en) => on_norm(&mut en.normal_export, func),
        Export::LevelExport(lev) => {
            lev.actors.iter_mut().for_each(&mut func);
            func(&mut lev.model);
            lev.model_components.iter_mut().for_each(&mut func);
            func(&mut lev.level_script);
            func(&mut lev.nav_list_start);
            func(&mut lev.nav_list_end);
            on_norm(&mut lev.normal_export, func)
        }
        Export::NormalExport(norm) => on_norm(norm, func),
        Export::PropertyExport(prop) => {
            uprop::on_uprop(&mut prop.property, func);
            on_norm(&mut prop.normal_export, func);
        }
        Export::RawExport(_) => (),
        Export::StringTableExport(str) => on_norm(&mut str.normal_export, func),
        Export::StructExport(struc) => on_struct(struc, func),
        Export::UserDefinedStructExport(uds) => {
            for prop in &mut uds.default_struct_instance {
                on_props(prop, func)
            }
            on_struct(&mut uds.struct_export, func)
        }
        Export::FunctionExport(fun) => on_struct(&mut fun.struct_export, func),
        Export::DataTableExport(data) => {
            for data in &mut data.table.data {
                for entry in &mut data.value {
                    on_props(entry, func);
                }
            }
            on_norm(&mut data.normal_export, func);
        }
        Export::WorldExport(world) => {
            func(&mut world.persistent_level);
            world.extra_objects.iter_mut().for_each(&mut func);
            world.streaming_levels.iter_mut().for_each(&mut func);
            on_norm(&mut world.normal_export, func);
        }
    }
}

/// on all possible export references
fn on_export_refs(export: &mut Export<PackageIndex>, mut func: impl FnMut(&mut PackageIndex)) {
    on_export(export, &mut func);
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

/// on all of an export's possible references to imports
fn on_import_refs(export: &mut Export<PackageIndex>, mut func: impl FnMut(&mut PackageIndex)) {
    on_export(export, &mut func);
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
