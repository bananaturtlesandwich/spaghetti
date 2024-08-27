use unreal_asset::exports::ExportBaseTrait;
use unreal_asset::kismet::*;
use unreal_asset::types::PackageIndex as Index;
use unreal_asset::Import;
use EExprToken as T;
use KismetExpression as K;
use KismetPropertyPointer as Pointer;

fn get_or_insert(import: Import, imports: &mut Vec<Import>) -> Index {
    match imports
        .iter()
        .position(|imp| {
            imp.class_package.eq_content(&import.class_package)
                && imp.class_name.eq_content(&import.class_name)
                && imp.object_name.eq_content(&import.object_name)
        })
        .map(|i| Index::new(-(i as i32 + 1)))
    {
        Some(i) => i,
        None => {
            let len = imports.len();
            imports.push(import);
            Index::new(-(len as i32 + 1))
        }
    }
}

pub fn init(
    mut name_map: std::cell::RefMut<unreal_asset::containers::NameMap>,
    asset: &mut unreal_asset::Asset<std::io::BufReader<std::fs::File>>,
) -> Vec<K> {
    // do this later
    let hook_folder = "";
    // currently don't know how many instructions this'll be
    let mut stack = Vec::with_capacity(4);
    let Some(ubergraph) = asset
        .asset_data
        .exports
        .iter()
        .position(|ex| {
            ex.get_base_export()
                .object_name
                .get_content(|name| name.starts_with("ExecuteUbergraph"))
        })
        .map(|i| Index::new(i as i32 + 1))
    else {
        eprintln!("couldn't find ubergraph");
        std::process::exit(0);
    };
    // reused names
    let coreuobject = name_map.add_fname("/Script/CoreUObject");
    let class_name = name_map.add_fname("Class");
    let function_name = name_map.add_fname("Function");
    let script_registry = get_or_insert(
        Import::new(
            coreuobject.clone(),
            name_map.add_fname("Package"),
            Index::new(0),
            name_map.add_fname("/Script/AssetRegistry"),
            false,
        ),
        &mut asset.imports,
    );
    let registry_helpers = get_or_insert(
        Import::new(
            coreuobject.clone(),
            class_name.clone(),
            script_registry,
            name_map.add_fname("AssetRegistryHelpers"),
            false,
        ),
        &mut asset.imports,
    );
    // get asset registry
    let registry = Pointer::from_new(FieldPath::new(
        vec![name_map.add_fname("CallFunc_GetAssetRegistry_ReturnValue")],
        ubergraph,
    ));
    let get_asset_registry = K::ExLet(ExLet {
        token: T::ExLet,
        value: registry.clone(),
        variable: Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: registry.clone(),
        })),
        expression: Box::new(K::ExCallMath(ExCallMath {
            token: T::ExCallMath,
            stack_node: get_or_insert(
                Import::new(
                    coreuobject.clone(),
                    function_name.clone(),
                    registry_helpers,
                    name_map.add_fname("GetAssetRegistry"),
                    false,
                ),
                &mut asset.imports,
            ),
            parameters: vec![],
        })),
    });
    stack.push(get_asset_registry.clone());
    let hook_folder_arr = Pointer::from_new(FieldPath {
        path: vec![name_map.add_fname("K2Node_MakeArray_Array")],
        resolved_owner: ubergraph,
    });
    // create local array for ScanPathsSynchronous
    stack.push(K::ExSetArray(ExSetArray {
        token: T::ExSetArray,
        assigning_property: Some(Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: hook_folder_arr.clone(),
        }))),
        array_inner_prop: None,
        elements: vec![K::ExStringConst(ExStringConst {
            token: T::ExLocalVariable,
            value: hook_folder.into(),
        })],
    }));
    // call scan paths on the asset registry interface
    stack.push(K::ExContext(ExContext {
        token: T::ExContext,
        object_expression: Box::new(K::ExInterfaceContext(ExInterfaceContext {
            token: T::ExInterfaceContext,
            interface_value: Box::new(K::ExLocalVariable(ExLocalVariable {
                token: T::ExLocalVariable,
                variable: registry.clone(),
            })),
        })),
        // what is this magical offset?
        offset: 25,
        r_value_pointer: Pointer::from_new(FieldPath {
            path: vec![],
            resolved_owner: Index::new(0),
        }),
        context_expression: Box::new(K::ExVirtualFunction(ExVirtualFunction {
            token: T::ExVirtualFunction,
            virtual_function_name: name_map.add_fname("ScanPathsSynchronous"),
            parameters: vec![
                K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: hook_folder_arr.clone(),
                }),
                K::ExTrue(ExTrue { token: T::ExTrue }),
                K::ExFalse(ExFalse { token: T::ExFalse }),
            ],
        })),
    }));
    // refresh our registry ref since paths were just scanned
    stack.push(get_asset_registry);
    // this local is generated by the virtual function
    let assets = Pointer::from_new(FieldPath::new(
        vec![name_map.add_fname("CallFunc_GetAssetsByPath_OutAssetData")],
        ubergraph,
    ));
    K::ExSetArray(ExSetArray {
        token: T::ExSetArray,
        assigning_property: Some(Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: assets.clone(),
        }))),
        array_inner_prop: None,
        elements: vec![],
    });
    // the return value of ScanPathsSynchronous isn't used
    /*
    K::ExLetBool(ExLetBool {
        token: T::ExLetBool,
        variable_expression: Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: assets.clone(),
        })),
        assignment_expression: Box::new(K::ExContext(ExContext {
            token: T::ExContext,
            object_expression: Box::new(K::ExInterfaceContext(ExInterfaceContext {
                token: T::ExInterfaceContext,
                interface_value: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: registry.clone(),
                })),
            })),
            offset: 34,
            r_value_pointer: assets.clone(),
            context_expression: Box::new(K::ExVirtualFunction(ExVirtualFunction {
                token: T::ExVirtualFunction,
                virtual_function_name: name_map.add_fname("GetAssetsByPath"),
                parameters: vec![
                    K::ExNameConst(ExNameConst {
                        token: T::ExNameConst,
                        value: name_map.add_fname(hook_folder),
                    }),
                    K::ExLocalVariable(ExLocalVariable {
                        token: T::ExLocalVariable,
                        variable: assets.clone(),
                    }),
                    K::ExTrue(ExTrue { token: T::ExTrue }),
                    K::ExFalse(ExFalse { token: T::ExFalse }),
                ],
            })),
        })),
    });
    */
    stack
}
