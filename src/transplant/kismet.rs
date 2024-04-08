use unreal_asset::kismet::*;
use unreal_asset::types::PackageIndex;
use KismetExpression as K;

fn on_pointer(pointer: &mut KismetPropertyPointer, func: &mut impl FnMut(&mut PackageIndex)) {
    if let Some(old) = pointer.old.as_mut() {
        func(old)
    }
}

pub fn on_kismet(inst: &mut K, func: &mut impl FnMut(&mut PackageIndex)) {
    match inst {
        K::ExLocalVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExInstanceVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExDefaultVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExReturn(ex) => on_kismet(&mut ex.return_expression, func),
        K::ExJump(_) => (),
        K::ExJumpIfNot(ex) => on_kismet(&mut ex.boolean_expression, func),
        K::ExAssert(ex) => on_kismet(&mut ex.assert_expression, func),
        K::ExNothing(_) => (),
        K::ExLet(ex) => {
            on_pointer(&mut ex.value, func);
            on_kismet(&mut ex.variable, func);
            on_kismet(&mut ex.expression, func);
        }
        K::ExClassContext(ex) => {
            on_kismet(&mut ex.object_expression, func);
            on_pointer(&mut ex.r_value_pointer, func);
            on_kismet(&mut ex.context_expression, func);
        }
        K::ExMetaCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target_expression, func);
        }
        K::ExLetBool(ex) => {
            on_kismet(&mut ex.variable_expression, func);
            on_kismet(&mut ex.assignment_expression, func);
        }
        K::ExEndParmValue(_) => (),
        K::ExEndFunctionParms(_) => (),
        K::ExSelf(_) => (),
        K::ExSkip(ex) => on_kismet(&mut ex.skip_expression, func),
        K::ExContext(ex) => {
            on_kismet(&mut ex.object_expression, func);
            on_pointer(&mut ex.r_value_pointer, func);
            on_kismet(&mut ex.context_expression, func);
        }
        K::ExContextFailSilent(ex) => {
            on_kismet(&mut ex.object_expression, func);
            on_pointer(&mut ex.r_value_pointer, func);
            on_kismet(&mut ex.context_expression, func);
        }
        K::ExVirtualFunction(ex) => {
            for par in &mut ex.parameters {
                on_kismet(par, func)
            }
        }
        K::ExFinalFunction(ex) => {
            func(&mut ex.stack_node);
            for par in &mut ex.parameters {
                on_kismet(par, func)
            }
        }
        K::ExIntConst(_) => (),
        K::ExFloatConst(_) => (),
        K::ExStringConst(_) => (),
        K::ExObjectConst(ex) => func(&mut ex.value),
        K::ExNameConst(_) => (),
        K::ExRotationConst(_) => (),
        K::ExVectorConst(_) => (),
        K::ExByteConst(_) => (),
        K::ExIntZero(_) => (),
        K::ExIntOne(_) => (),
        K::ExTrue(_) => (),
        K::ExFalse(_) => (),
        K::ExTextConst(_) => (),
        K::ExNoObject(_) => (),
        K::ExTransformConst(_) => (),
        K::ExIntConstByte(_) => (),
        K::ExNoInterface(_) => (),
        K::ExDynamicCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target_expression, func);
        }
        K::ExStructConst(ex) => {
            func(&mut ex.struct_value);
            for inst in &mut ex.value {
                on_kismet(inst, func)
            }
        }
        K::ExEndStructConst(_) => (),
        K::ExSetArray(ex) => {
            if let Some(ap) = ex.assigning_property.as_mut() {
                on_kismet(ap, func)
            }
            if let Some(inner) = &mut ex.array_inner_prop {
                func(inner)
            }
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExEndArray(_) => (),
        K::ExPropertyConst(ex) => on_pointer(&mut ex.property, func),
        K::ExUnicodeStringConst(_) => (),
        K::ExInt64Const(_) => (),
        K::ExUInt64Const(_) => (),
        K::ExDoubleConst(_) => (),
        K::ExPrimitiveCast(ex) => on_kismet(&mut ex.target, func),
        K::ExSetSet(ex) => {
            on_kismet(&mut ex.set_property, func);
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExEndSet(_) => (),
        K::ExSetMap(ex) => {
            on_kismet(&mut ex.map_property, func);
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExEndMap(_) => (),
        K::ExSetConst(ex) => {
            on_pointer(&mut ex.inner_property, func);
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExEndSetConst(_) => (),
        K::ExMapConst(ex) => {
            on_pointer(&mut ex.key_property, func);
            on_pointer(&mut ex.value_property, func);
        }
        K::ExEndMapConst(_) => (),
        K::ExStructMemberContext(ex) => {
            on_pointer(&mut ex.struct_member_expression, func);
            on_kismet(&mut ex.struct_expression, func);
        }
        K::ExLetMulticastDelegate(ex) => {
            on_kismet(&mut ex.variable_expression, func);
            on_kismet(&mut ex.assignment_expression, func);
        }
        K::ExLetDelegate(ex) => {
            on_kismet(&mut ex.variable_expression, func);
            on_kismet(&mut ex.assignment_expression, func);
        }
        K::ExLocalVirtualFunction(ex) => {
            for par in &mut ex.parameters {
                on_kismet(par, func)
            }
        }
        K::ExLocalFinalFunction(ex) => {
            func(&mut ex.stack_node);
            for par in &mut ex.parameters {
                on_kismet(par, func)
            }
        }
        K::ExLocalOutVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExDeprecatedOp4A(_) => (),
        K::ExInstanceDelegate(ex) => (),
        K::ExPushExecutionFlow(ex) => (),
        K::ExPopExecutionFlow(ex) => (),
        K::ExComputedJump(ex) => on_kismet(&mut ex.code_offset_expression, func),
        K::ExPopExecutionFlowIfNot(ex) => on_kismet(&mut ex.boolean_expression, func),
        K::ExBreakpoint(ex) => (),
        K::ExInterfaceContext(ex) => on_kismet(&mut ex.interface_value, func),
        K::ExObjToInterfaceCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target, func);
        }
        K::ExEndOfScript(ex) => (),
        K::ExCrossInterfaceCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target, func);
        }
        K::ExInterfaceToObjCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target, func);
        }
        K::ExWireTracepoint(ex) => (),
        K::ExSkipOffsetConst(ex) => (),
        K::ExAddMulticastDelegate(ex) => {
            on_kismet(&mut ex.delegate, func);
            on_kismet(&mut ex.delegate_to_add, func);
        }
        K::ExClearMulticastDelegate(ex) => on_kismet(&mut ex.delegate_to_clear, func),
        K::ExTracepoint(ex) => (),
        K::ExLetObj(ex) => {
            on_kismet(&mut ex.variable_expression, func);
            on_kismet(&mut ex.assignment_expression, func);
        }
        K::ExLetWeakObjPtr(ex) => {
            on_kismet(&mut ex.variable_expression, func);
            on_kismet(&mut ex.assignment_expression, func);
        }
        K::ExBindDelegate(ex) => {
            on_kismet(&mut ex.delegate, func);
            on_kismet(&mut ex.object_term, func);
        }
        K::ExRemoveMulticastDelegate(ex) => {
            on_kismet(&mut ex.delegate, func);
            on_kismet(&mut ex.delegate_to_add, func);
        }
        K::ExCallMulticastDelegate(ex) => {
            func(&mut ex.stack_node);
            for par in &mut ex.parameters {
                on_kismet(par, func)
            }
            on_kismet(&mut ex.delegate, func);
        }
        K::ExLetValueOnPersistentFrame(ex) => {
            on_pointer(&mut ex.destination_property, func);
            on_kismet(&mut ex.assignment_expression, func)
        }
        K::ExArrayConst(ex) => {
            on_pointer(&mut ex.inner_property, func);
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExEndArrayConst(ex) => (),
        K::ExSoftObjectConst(ex) => on_kismet(&mut ex.value, func),
        K::ExCallMath(ex) => {
            func(&mut ex.stack_node);
            for par in &mut ex.parameters {
                on_kismet(par, func)
            }
        }
        K::ExSwitchValue(ex) => {
            on_kismet(&mut ex.index_term, func);
            on_kismet(&mut ex.default_term, func);
            for case in &mut ex.cases {
                on_kismet(&mut case.case_index_value_term, func);
                on_kismet(&mut case.case_term, func);
            }
        }
        K::ExInstrumentationEvent(ex) => (),
        K::ExArrayGetByRef(ex) => {
            on_kismet(&mut ex.array_variable, func);
            on_kismet(&mut ex.array_index, func);
        }
        K::ExClassSparseDataVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExFieldPathConst(ex) => on_kismet(&mut ex.value, func),
    }
}
