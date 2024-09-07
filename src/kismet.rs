use unreal_asset::exports::ExportBaseTrait;
use unreal_asset::kismet::*;
use unreal_asset::types::PackageIndex as Index;
use unreal_asset::Import;
use EExprToken as T;
use KismetExpression as K;
use KismetPropertyPointer as Pointer;

// need to add local variables to the loaded properties
pub fn hook(
    function: &mut unreal_asset::exports::FunctionExport<Index>,
    this: Index,
    name_map: &mut unreal_asset::containers::SharedResource<unreal_asset::containers::NameMap>,
    blueprint: &mut unreal_asset::Asset<std::io::BufReader<std::fs::File>>,
) -> Vec<K> {
    let init = function.get_base_export().object_name == "ReceiveBeginPlay";
    macro_rules! name {
        ($name: expr) => {
            name_map.get_mut().add_fname($name)
        };
    }
    // do this later
    let hook_folder = "";
    let hook_path = "";
    let hook_name = "";
    let function = "";
    // currently don't know how many instructions this'll be
    let mut stack = vec![];
    let mut offset = 0;
    let obj = blueprint.asset_data.object_version_ue5;
    macro_rules! push {
        ($inst: expr) => {{
            offset += size(&obj, &$inst);
            stack.push($inst);
        }};
    }
    let mut get_or_insert = |import: Import| -> Index {
        match blueprint
            .imports
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
                let len = blueprint.imports.len();
                blueprint.imports.push(import);
                Index::new(-(len as i32 + 1))
            }
        }
        // need to add to create before serialisation deps
    };
    let null = Pointer::from_new(FieldPath::new(vec![], Index::new(0)));
    let coreuobject_name = name!("/Script/CoreUObject");
    let package_name = name!("Package");
    let script_hook_interface = get_or_insert(Import::new(
        coreuobject_name.clone(),
        package_name.clone(),
        Index::new(0),
        name!(hook_path),
        false,
    ));
    let engine_name = name!("/Script/Engine");
    let blueprint_generated_class_name = name!("BlueprintGeneratedClass");
    let hook_interface = get_or_insert(Import::new(
        engine_name.clone(),
        blueprint_generated_class_name.clone(),
        script_hook_interface,
        name!(hook_name),
        false,
    ));
    let hooks_name = name!("hooks");
    let none_name = name!("None");
    let Some(class) = blueprint
        .asset_data
        .exports
        .iter_mut()
        .position(|ex| match ex {
            unreal_asset::Export::ClassExport(class) => {
                use unreal_asset::enums::EArrayDim as D;
                use unreal_asset::enums::ELifetimeCondition as L;
                use unreal_asset::flags::EObjectFlags as O;
                use unreal_asset::flags::EPropertyFlags as P;
                use unreal_asset::fproperty::*;
                // add the hooks variable
                if init {
                    class
                        .struct_export
                        .loaded_properties
                        .push(FProperty::FArrayProperty(FArrayProperty {
                            generic_property: FGenericProperty {
                                name: hooks_name.clone(),
                                flags: O::RF_PUBLIC | O::RF_LOAD_COMPLETED,
                                array_dim: D::TArray,
                                element_size: 16,
                                property_flags: P::CPF_EDIT
                                    | P::CPF_BLUEPRINT_VISIBLE
                                    | P::CPF_DISABLE_EDIT_ON_INSTANCE,
                                rep_index: 0,
                                rep_notify_func: none_name.clone(),
                                blueprint_replication_condition: L::CondNone,
                                serialized_type: None,
                            },
                            inner: Box::new(FProperty::FInterfaceProperty(FInterfaceProperty {
                                generic_property: FGenericProperty {
                                    name: hooks_name.clone(),
                                    flags: O::RF_PUBLIC,
                                    array_dim: D::TArray,
                                    element_size: 16,
                                    property_flags: P::CPF_NONE,
                                    rep_index: 0,
                                    rep_notify_func: none_name.clone(),
                                    blueprint_replication_condition: L::CondNone,
                                    serialized_type: None,
                                },
                                interface_class: hook_interface,
                            })),
                        }));
                }
                true
            }
            _ => false,
        })
        .map(|i| Index::new(i as i32 + 1))
    else {
        eprintln!("couldn't find ubergraph");
        std::process::exit(0);
    };
    let class_name = name!("Class");
    let script_registry = get_or_insert(Import::new(
        coreuobject_name.clone(),
        package_name.clone(),
        Index::new(0),
        name!("/Script/AssetRegistry"),
        false,
    ));
    let registry_helpers = get_or_insert(Import::new(
        coreuobject_name.clone(),
        class_name.clone(),
        script_registry,
        name!("AssetRegistryHelpers"),
        false,
    ));
    // get asset registry
    let registry = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_GetAssetRegistry_ReturnValue")],
        this,
    ));
    let function_name = name!("Function");
    let get_asset_registry = K::ExLet(ExLet {
        token: T::ExLet,
        value: registry.clone(),
        variable: Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: registry.clone(),
        })),
        expression: Box::new(K::ExCallMath(ExCallMath {
            token: T::ExCallMath,
            stack_node: get_or_insert(Import::new(
                coreuobject_name.clone(),
                function_name.clone(),
                registry_helpers,
                name!("GetAssetRegistry"),
                false,
            )),
            parameters: vec![],
        })),
    });
    push!(get_asset_registry.clone());
    let hook_folder_arr =
        Pointer::from_new(FieldPath::new(vec![name!("K2Node_MakeArray_Array")], this));
    // create local array for ScanPathsSynchronous
    push!(K::ExSetArray(ExSetArray {
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
    // call ScanPathsSynchronous on the asset registry interface
    push!(K::ExContext(ExContext {
        token: T::ExContext,
        object_expression: Box::new(K::ExInterfaceContext(ExInterfaceContext {
            token: T::ExInterfaceContext,
            interface_value: Box::new(K::ExLocalVariable(ExLocalVariable {
                token: T::ExLocalVariable,
                variable: registry.clone(),
            })),
        })),
        offset: 25,
        r_value_pointer: null.clone(),
        context_expression: Box::new(K::ExVirtualFunction(ExVirtualFunction {
            token: T::ExVirtualFunction,
            virtual_function_name: name!("ScanPathsSynchronous"),
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
    // refresh our registry since paths were just scanned
    push!(get_asset_registry);
    // i think this local is generated by the virtual function which is why there's no let expression
    let assets = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_GetAssetsByPath_OutAssetData")],
        this,
    ));
    push!(K::ExSetArray(ExSetArray {
        token: T::ExSetArray,
        assigning_property: Some(Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: assets.clone(),
        }))),
        array_inner_prop: None,
        elements: vec![],
    }));
    // return value isn't used so probably doesn't need to be set

    // okay time to set up the for loop stuff
    let counter = Pointer::from_new(FieldPath::new(vec![name!("counter")], this));
    let is_less = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_Less_IntInt_ReturnValue")],
        this,
    ));
    let len = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_Array_Length_ReturnValue")],
        this,
    ));
    let script_engine = get_or_insert(Import::new(
        coreuobject_name.clone(),
        package_name.clone(),
        Index::new(0),
        engine_name.clone(),
        false,
    ));
    let kismet_math_library = get_or_insert(Import::new(
        coreuobject_name.clone(),
        class_name.clone(),
        script_engine,
        name!("Less_IntInt"),
        false,
    ));
    let kismet_array_library_name = name!("KismetArrayLibrary");
    let default_kismet_array_library = get_or_insert(Import::new(
        engine_name.clone(),
        kismet_array_library_name.clone(),
        script_engine,
        name!("Default__KismetArrayLibrary"),
        false,
    ));
    let kismet_array_library = get_or_insert(Import::new(
        coreuobject_name.clone(),
        class_name.clone(),
        script_engine,
        kismet_array_library_name.clone(),
        false,
    ));
    let hooks = Pointer::from_new(FieldPath::new(vec![hooks_name.clone()], class));
    push!(K::ExLet(ExLet {
        token: T::ExLet,
        value: len.clone(),
        variable: Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: len.clone(),
        })),
        expression: Box::new(K::ExContext(ExContext {
            token: T::ExContext,
            object_expression: Box::new(K::ExObjectConst(ExObjectConst {
                token: T::ExObjectConst,
                value: default_kismet_array_library,
            })),
            offset: 19,
            r_value_pointer: len.clone(),
            context_expression: Box::new(K::ExFinalFunction(ExFinalFunction {
                token: T::ExFinalFunction,
                stack_node: get_or_insert(Import::new(
                    coreuobject_name.clone(),
                    function_name.clone(),
                    kismet_array_library,
                    name!("Array_Length"),
                    false,
                ),),
                parameters: vec![K::ExInstanceVariable(ExInstanceVariable {
                    token: T::ExInstanceVariable,
                    variable: hooks.clone(),
                })],
            })),
        })),
    }));
    let incremented = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_Add_IntInt_ReturnValue")],
        this,
    ));
    // no need to refresh the length of the array since it doesn't change
    macro_rules! for_loop {
        ($len: expr, $inst: expr) => {{
            // reset counter
            push!(K::ExLet(ExLet {
                token: T::ExLet,
                value: counter.clone(),
                variable: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: counter.clone(),
                })),
                expression: Box::new(K::ExIntConst(ExIntConst {
                    token: T::ExIntConst,
                    value: 0,
                })),
            }));
            let start = offset;
            push!(K::ExLetBool(ExLetBool {
                token: T::ExLetBool,
                variable_expression: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: is_less.clone(),
                })),
                assignment_expression: Box::new(K::ExCallMath(ExCallMath {
                    token: T::ExCallMath,
                    stack_node: get_or_insert(
                        Import::new(
                            coreuobject_name.clone(),
                            function_name.clone(),
                            kismet_math_library,
                            name!("Less_IntInt"),
                            false,
                        ),
                    ),
                    parameters: vec![
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: counter.clone(),
                        }),
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: $len.clone(),
                        }),
                    ],
                })),
            }));
            // ExecutionFlow involves offsets i don't want to deal with so just go straight to increment

            // declare ending instructions so we can calculate offset to end of loop
            let increment = K::ExLet(ExLet {
                token: T::ExLet,
                value: incremented.clone(),
                variable: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: incremented.clone(),
                })),
                expression: Box::new(K::ExCallMath(ExCallMath {
                    token: T::ExCallMath,
                    stack_node: get_or_insert(
                        Import::new(
                            coreuobject_name.clone(),
                            function_name.clone(),
                            kismet_math_library,
                            name!("Add_IntInt"),
                            false,
                        ),
                    ),
                    parameters: vec![
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: counter.clone(),
                        }),
                        K::ExIntConst(ExIntConst {
                            token: T::ExIntConst,
                            value: 1,
                        }),
                    ],
                })),
            });
            let update = K::ExLet(ExLet {
                token: T::ExLet,
                value: counter.clone(),
                variable: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: counter.clone(),
                })),
                expression: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: incremented.clone(),
                })),
            });
            let jump = K::ExJump(ExJump {
                token: T::ExJump,
                code_offset: start,
            });
            let inst_end = offset
                + 1 + 4 // jumpifnot
                + 1 + 8 // jumpifnot boolean expression is localvariable
                + 1 + 4 // pushexecutionflow
                + $inst.iter().map(|inst| size(&obj, inst)).sum::<u32>()
                + 1; // popexecutionflow
            push!(K::ExJumpIfNot(ExJumpIfNot {
                token: T::ExJumpIfNot,
                code_offset: inst_end + size(&obj, &increment) + size(&obj, &update) + size(&obj, &jump),
                boolean_expression: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: is_less.clone(),
                })),
            }));
            push!(K::ExPushExecutionFlow(ExPushExecutionFlow {
                token: T::ExPushExecutionFlow,
                pushing_address: inst_end
            }));
            for inst in $inst {
                push!(inst)
            }
            push!(K::ExPopExecutionFlow(ExPopExecutionFlow {
               token: T::ExPopExecutionFlow,
            }));
            push!(increment);
            push!(update);
            push!(jump);
        }};
    }
    let len = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_Array_Length_ReturnValue")],
        this,
    ));
    push!(K::ExLet(ExLet {
        token: T::ExLet,
        value: len.clone(),
        variable: Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: len.clone(),
        })),
        expression: Box::new(K::ExContext(ExContext {
            token: T::ExContext,
            object_expression: Box::new(K::ExObjectConst(ExObjectConst {
                token: T::ExObjectConst,
                value: default_kismet_array_library,
            })),
            offset: 19,
            r_value_pointer: len.clone(),
            context_expression: Box::new(K::ExFinalFunction(ExFinalFunction {
                token: T::ExFinalFunction,
                stack_node: get_or_insert(Import::new(
                    coreuobject_name.clone(),
                    function_name.clone(),
                    kismet_array_library,
                    name!("Array_Length"),
                    false,
                ),),
                parameters: vec![K::ExInstanceVariable(ExInstanceVariable {
                    token: T::ExInstanceVariable,
                    variable: assets.clone(),
                })],
            })),
        })),
    }));
    let item = Pointer::from_new(FieldPath::new(vec![name!("CallFunc_Array_Get_Item")], this));
    let get_asset_name = name!("GetAsset");
    let asset = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_GetAsset_ReturnValue")],
        this,
    ));
    let cast = Pointer::from_new(FieldPath::new(
        vec![name!(&format!("K2Node_DynamicCast_As{hook_name}"))],
        this,
    ));
    let cast_success = Pointer::from_new(FieldPath::new(
        vec![name!("K2Node_DynamicCast_bSuccess")],
        this,
    ));
    let array_added = Pointer::from_new(FieldPath::new(
        vec![name!("CallFunc_Array_Add_ReturnValue")],
        this,
    ));
    let array_add = get_or_insert(Import::new(
        coreuobject_name.clone(),
        function_name.clone(),
        kismet_array_library,
        name!("Array_Add"),
        false,
    ));
    for_loop!(
        len,
        vec![
            K::ExContext(ExContext {
                token: T::ExContext,
                object_expression: Box::new(K::ExObjectConst(ExObjectConst {
                    token: T::ExObjectConst,
                    value: default_kismet_array_library,
                })),
                offset: 37,
                r_value_pointer: null.clone(),
                context_expression: Box::new(K::ExFinalFunction(ExFinalFunction {
                    token: T::ExFinalFunction,
                    stack_node: get_or_insert(Import::new(
                        coreuobject_name.clone(),
                        function_name.clone(),
                        kismet_array_library,
                        name!("Array_Get"),
                        false,
                    ),),
                    parameters: vec![
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: assets.clone(),
                        }),
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: counter.clone(),
                        }),
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: item.clone(),
                        }),
                    ],
                })),
            }),
            K::ExLetObj(ExLetObj {
                token: T::ExLetObj,
                variable_expression: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: asset.clone(),
                })),
                assignment_expression: Box::new(K::ExCallMath(ExCallMath {
                    token: T::ExCallMath,
                    stack_node: get_or_insert(Import::new(
                        coreuobject_name.clone(),
                        function_name.clone(),
                        registry_helpers,
                        get_asset_name.clone(),
                        false,
                    ),),
                    parameters: vec![K::ExLocalVariable(ExLocalVariable {
                        token: T::ExLocalVariable,
                        variable: item.clone(),
                    })],
                })),
            }),
            K::ExLet(ExLet {
                token: T::ExLet,
                value: null.clone(),
                variable: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: cast.clone(),
                })),
                expression: Box::new(K::ExObjToInterfaceCast(ExObjToInterfaceCast {
                    token: T::ExObjToInterfaceCast,
                    class_ptr: hook_interface,
                    target: Box::new(K::ExLocalVariable(ExLocalVariable {
                        token: T::ExLocalVariable,
                        variable: asset.clone(),
                    }))
                })),
            }),
            K::ExLet(ExLet {
                token: T::ExLet,
                value: null.clone(),
                variable: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: cast_success.clone(),
                })),
                expression: Box::new(K::ExPrimitiveCast(ExPrimitiveCast {
                    token: T::ExPrimitiveCast,
                    // need to change this for lower versions
                    conversion_type: ECastToken::InterfaceToBool2,
                    target: Box::new(K::ExLocalVariable(ExLocalVariable {
                        token: T::ExLocalVariable,
                        variable: cast.clone(),
                    }))
                })),
            }),
            K::ExPopExecutionFlowIfNot(ExPopExecutionFlowIfNot {
                token: T::ExPopExecutionFlowIfNot,
                boolean_expression: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: cast_success.clone(),
                }))
            }),
            K::ExLet(ExLet {
                token: T::ExLet,
                value: array_added.clone(),
                variable: Box::new(K::ExLocalVariable(ExLocalVariable {
                    token: T::ExLocalVariable,
                    variable: array_added.clone(),
                })),
                expression: Box::new(K::ExContext(ExContext {
                    token: T::ExContext,
                    object_expression: Box::new(K::ExObjectConst(ExObjectConst {
                        token: T::ExObjectConst,
                        value: default_kismet_array_library
                    })),
                    offset: 28,
                    r_value_pointer: array_added.clone(),
                    context_expression: Box::new(K::ExFinalFunction(ExFinalFunction {
                        token: T::ExFinalFunction,
                        stack_node: array_add,
                        parameters: vec![
                            K::ExInstanceVariable(ExInstanceVariable {
                                token: T::ExInstanceVariable,
                                variable: hooks.clone()
                            }),
                            K::ExLocalVariable(ExLocalVariable {
                                token: T::ExLocalVariable,
                                variable: cast.clone()
                            })
                        ]
                    }))
                }))
            }),
        ]
    );
    push!(K::ExLet(ExLet {
        token: T::ExLet,
        value: len.clone(),
        variable: Box::new(K::ExLocalVariable(ExLocalVariable {
            token: T::ExLocalVariable,
            variable: len.clone(),
        })),
        expression: Box::new(K::ExContext(ExContext {
            token: T::ExContext,
            object_expression: Box::new(K::ExObjectConst(ExObjectConst {
                token: T::ExObjectConst,
                value: default_kismet_array_library,
            })),
            offset: 19,
            r_value_pointer: len.clone(),
            context_expression: Box::new(K::ExFinalFunction(ExFinalFunction {
                token: T::ExFinalFunction,
                stack_node: get_or_insert(Import::new(
                    coreuobject_name.clone(),
                    function_name.clone(),
                    kismet_array_library,
                    name!("Array_Length"),
                    false,
                ),),
                parameters: vec![K::ExInstanceVariable(ExInstanceVariable {
                    token: T::ExInstanceVariable,
                    variable: hooks.clone(),
                })],
            })),
        })),
    }));
    let pre_function = name!(&format!("pre_{function}"));
    for_loop!(
        len.clone(),
        vec![
            K::ExContext(ExContext {
                token: T::ExContext,
                object_expression: Box::new(K::ExObjectConst(ExObjectConst {
                    token: T::ExObjectConst,
                    value: default_kismet_array_library,
                })),
                offset: 37,
                r_value_pointer: null.clone(),
                context_expression: Box::new(K::ExFinalFunction(ExFinalFunction {
                    token: T::ExFinalFunction,
                    stack_node: get_or_insert(Import::new(
                        coreuobject_name.clone(),
                        function_name.clone(),
                        kismet_array_library,
                        name!("Array_Get"),
                        false,
                    ),),
                    parameters: vec![
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: hooks.clone(),
                        }),
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: counter.clone(),
                        }),
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: item.clone(),
                        }),
                    ],
                })),
            }),
            K::ExContext(ExContext {
                token: T::ExContext,
                object_expression: Box::new(K::ExInterfaceContext(ExInterfaceContext {
                    token: T::ExInterfaceContext,
                    interface_value: Box::new(K::ExLocalVariable(ExLocalVariable {
                        token: T::ExLocalVariable,
                        variable: item.clone(),
                    })),
                })),
                offset: 15,
                r_value_pointer: null.clone(),
                context_expression: Box::new(K::ExLocalVirtualFunction(ExLocalVirtualFunction {
                    token: T::ExLocalVirtualFunction,
                    virtual_function_name: pre_function.clone(),
                    parameters: vec![
                        K::ExSelf(ExSelf { token: T::ExSelf }),
                        // other params
                    ]
                }))
            })
        ]
    );
    push!(K::ExLocalVirtualFunction(ExLocalVirtualFunction {
        token: T::ExLocalVirtualFunction,
        virtual_function_name: name!(&format!("orig_{function}")),
        parameters: vec![
            // other params
        ]
    }));
    let post_function = name!(&format!("post_{function}"));
    for_loop!(
        len.clone(),
        vec![
            K::ExContext(ExContext {
                token: T::ExContext,
                object_expression: Box::new(K::ExObjectConst(ExObjectConst {
                    token: T::ExObjectConst,
                    value: default_kismet_array_library,
                })),
                offset: 37,
                r_value_pointer: null.clone(),
                context_expression: Box::new(K::ExFinalFunction(ExFinalFunction {
                    token: T::ExFinalFunction,
                    stack_node: get_or_insert(Import::new(
                        coreuobject_name.clone(),
                        function_name.clone(),
                        kismet_array_library,
                        name!("Array_Get"),
                        false,
                    ),),
                    parameters: vec![
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: hooks.clone(),
                        }),
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: counter.clone(),
                        }),
                        K::ExLocalVariable(ExLocalVariable {
                            token: T::ExLocalVariable,
                            variable: item.clone(),
                        }),
                    ],
                })),
            }),
            K::ExContext(ExContext {
                token: T::ExContext,
                object_expression: Box::new(K::ExInterfaceContext(ExInterfaceContext {
                    token: T::ExInterfaceContext,
                    interface_value: Box::new(K::ExLocalVariable(ExLocalVariable {
                        token: T::ExLocalVariable,
                        variable: item.clone(),
                    })),
                })),
                offset: 15,
                r_value_pointer: null.clone(),
                context_expression: Box::new(K::ExLocalVirtualFunction(ExLocalVirtualFunction {
                    token: T::ExLocalVirtualFunction,
                    virtual_function_name: post_function.clone(),
                    parameters: vec![
                        K::ExSelf(ExSelf { token: T::ExSelf }),
                        // other params
                    ]
                }))
            })
        ]
    );
    // to suppress warnings about not using offset
    let _ = offset;
    stack
}

#[test]
// confirming that the offset field in ExContext is the size of the context expression
fn offset() {
    let name = unreal_asset::types::FName::new_dummy("".into(), 0);
    let named = Pointer::from_new(FieldPath::new(vec![name.clone()], Index::new(0)));
    assert_eq!(
        size(
            &unreal_asset::object_version::ObjectVersionUE5::ADD_SOFTOBJECTPATH_LIST,
            &K::ExVirtualFunction(ExVirtualFunction {
                token: T::ExVirtualFunction,
                virtual_function_name: name,
                parameters: vec![
                    K::ExLocalVariable(ExLocalVariable {
                        token: T::ExLocalVariable,
                        variable: named.clone(),
                    }),
                    K::ExTrue(ExTrue { token: T::ExTrue }),
                    K::ExFalse(ExFalse { token: T::ExFalse }),
                ],
            }),
        ),
        25
    );
}

fn size(obj: &unreal_asset::object_version::ObjectVersionUE5, inst: &K) -> u32 {
    use unreal_asset::object_version::ObjectVersionUE5;
    1 + match inst {
        K::ExPushExecutionFlow(_) => 4,
        K::ExComputedJump(e) => size(obj, &e.code_offset_expression),
        K::ExJump(_) => 4,
        K::ExJumpIfNot(e) => 4 + size(obj, &e.boolean_expression),
        K::ExLocalVariable(_) => 8,
        K::ExDefaultVariable(_) => 8,
        K::ExObjToInterfaceCast(e) => 8 + size(obj, &e.target),
        K::ExLet(e) => 8 + size(obj, &e.variable) + size(obj, &e.expression),
        K::ExLetObj(e) => size(obj, &e.variable_expression) + size(obj, &e.assignment_expression),
        K::ExLetBool(e) => size(obj, &e.variable_expression) + size(obj, &e.assignment_expression),
        K::ExLetWeakObjPtr(e) => {
            size(obj, &e.variable_expression) + size(obj, &e.assignment_expression)
        }
        K::ExLetValueOnPersistentFrame(e) => 8 + size(obj, &e.assignment_expression),
        K::ExStructMemberContext(e) => 8 + size(obj, &e.struct_expression),
        K::ExMetaCast(e) => 8 + size(obj, &e.target_expression),
        K::ExDynamicCast(e) => 8 + size(obj, &e.target_expression),
        K::ExPrimitiveCast(e) => {
            1 + match e.conversion_type {
                ECastToken::ObjectToInterface => 8,
                /* TODO InterfaceClass */
                _ => 0,
            } + size(obj, &e.target)
        }
        K::ExPopExecutionFlow(_) => 0,
        K::ExPopExecutionFlowIfNot(e) => size(obj, &e.boolean_expression),
        K::ExCallMath(e) => 8 + e.parameters.iter().map(|e| size(obj, e)).sum::<u32>() + 1,
        K::ExSwitchValue(e) => {
            6 + size(obj, &e.index_term)
                + e.cases
                    .iter()
                    .map(|e| size(obj, &e.case_index_value_term) + 4 + size(obj, &e.case_term))
                    .sum::<u32>()
                + size(obj, &e.default_term)
        }
        K::ExSelf(_) => 0,
        K::ExTextConst(e) => {
            1 + match e.value.text_literal_type {
                EBlueprintTextLiteralType::Empty => 0,
                EBlueprintTextLiteralType::LocalizedText => {
                    e.value
                        .localized_source
                        .as_ref()
                        .map(|l| size(obj, l))
                        .unwrap_or_default()
                        + e.value
                            .localized_key
                            .as_ref()
                            .map(|l| size(obj, l))
                            .unwrap_or_default()
                        + e.value
                            .localized_namespace
                            .as_ref()
                            .map(|l| size(obj, l))
                            .unwrap_or_default()
                }
                EBlueprintTextLiteralType::InvariantText => e
                    .value
                    .invariant_literal_string
                    .as_ref()
                    .map(|l| size(obj, l))
                    .unwrap_or_default(),
                EBlueprintTextLiteralType::LiteralString => e
                    .value
                    .literal_string
                    .as_ref()
                    .map(|l| size(obj, l))
                    .unwrap_or_default(),
                EBlueprintTextLiteralType::StringTableEntry => {
                    8 + e
                        .value
                        .string_table_id
                        .as_ref()
                        .map(|l| size(obj, l))
                        .unwrap_or_default()
                        + e.value
                            .string_table_key
                            .as_ref()
                            .map(|l| size(obj, l))
                            .unwrap_or_default()
                }
            }
        }
        K::ExObjectConst(_) => 8,
        K::ExVectorConst(_) => match obj >= &ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => 24,
            false => 12,
        },
        K::ExRotationConst(_) => match obj >= &ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => 24,
            false => 12,
        },
        K::ExTransformConst(_) => match obj >= &ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => 80,
            false => 40,
        },
        K::ExContext(e) => {
            size(obj, &e.object_expression) + 4 + 8 + size(obj, &e.context_expression)
        }
        K::ExCallMulticastDelegate(e) => {
            8 + size(obj, &e.delegate) + e.parameters.iter().map(|e| size(obj, e)).sum::<u32>() + 1
        }
        K::ExLocalFinalFunction(e) => {
            8 + e.parameters.iter().map(|e| size(obj, e)).sum::<u32>() + 1
        }
        K::ExFinalFunction(e) => 8 + e.parameters.iter().map(|e| size(obj, e)).sum::<u32>() + 1,
        K::ExLocalVirtualFunction(e) => {
            12 + e.parameters.iter().map(|e| size(obj, e)).sum::<u32>() + 1
        }
        K::ExVirtualFunction(e) => 12 + e.parameters.iter().map(|e| size(obj, e)).sum::<u32>() + 1,
        K::ExInstanceVariable(_) => 8,
        K::ExAddMulticastDelegate(e) => size(obj, &e.delegate) + size(obj, &e.delegate_to_add),
        K::ExRemoveMulticastDelegate(e) => size(obj, &e.delegate) + size(obj, &e.delegate_to_add),
        K::ExClearMulticastDelegate(e) => size(obj, &e.delegate_to_clear),
        K::ExBindDelegate(e) => 12 + size(obj, &e.delegate) + size(obj, &e.object_term),
        K::ExStructConst(e) => 8 + 4 + e.value.iter().map(|e| size(obj, e)).sum::<u32>() + 1,
        K::ExSetArray(e) => {
            e.assigning_property
                .as_ref()
                .map(|l| size(obj, l))
                .unwrap_or_default()
                + e.elements.iter().map(|e| size(obj, e)).sum::<u32>()
                + 1
        }
        K::ExSetMap(e) => {
            size(obj, &e.map_property)
                + 4
                + e.elements.iter().map(|e| size(obj, e)).sum::<u32>()
                + 1
        }
        K::ExSetSet(e) => {
            size(obj, &e.set_property)
                + 4
                + e.elements.iter().map(|e| size(obj, e)).sum::<u32>()
                + 1
        }
        K::ExSoftObjectConst(e) => size(obj, &e.value),
        K::ExByteConst(_) => 1,
        K::ExIntConst(_) => 4,
        K::ExFloatConst(_) => 4,
        K::ExInt64Const(_) => 8,
        K::ExUInt64Const(_) => 8,
        K::ExNameConst(_) => 12,
        K::ExStringConst(e) => e.value.len() as u32 + 1,
        K::ExUnicodeStringConst(e) => 2 * (e.value.len() as u32 + 1),
        K::ExSkipOffsetConst(_) => 4,
        K::ExArrayConst(e) => 12 + e.elements.iter().map(|e| size(obj, e)).sum::<u32>() + 1,
        K::ExReturn(e) => size(obj, &e.return_expression),
        K::ExLocalOutVariable(_) => 8,
        K::ExInterfaceContext(e) => size(obj, &e.interface_value),
        K::ExInterfaceToObjCast(e) => 8 + size(obj, &e.target),
        K::ExArrayGetByRef(e) => size(obj, &e.array_variable) + size(obj, &e.array_index),
        K::ExTrue(_) => 0,
        K::ExFalse(_) => 0,
        K::ExNothing(_) => 0,
        K::ExNoObject(_) => 0,
        K::ExEndOfScript(_) => 0,
        K::ExTracepoint(_) => 0,
        K::ExWireTracepoint(_) => 0,
        // none of the procedurally written kismet isn't here
        _ => todo!(),
    }
}
