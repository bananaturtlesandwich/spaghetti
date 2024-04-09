use unreal_asset::kismet::*;
use unreal_asset::types::PackageIndex;
use KismetExpression as K;

fn on_pointer(pointer: &mut KismetPropertyPointer, func: &mut impl FnMut(&mut PackageIndex)) {
    if let Some(old) = pointer.old.as_mut() {
        func(old)
    }
    if let Some(new) = pointer.new.as_mut() {
        func(&mut new.resolved_owner);
    }
}

pub fn on_kismet(inst: &mut K, func: &mut impl FnMut(&mut PackageIndex)) {
    match inst {
        K::ExLocalVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExInstanceVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExDefaultVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExReturn(ex) => on_kismet(&mut ex.return_expression, func),
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
        K::ExObjectConst(ex) => func(&mut ex.value),
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
        K::ExPropertyConst(ex) => on_pointer(&mut ex.property, func),
        K::ExPrimitiveCast(ex) => on_kismet(&mut ex.target, func),
        K::ExSetSet(ex) => {
            on_kismet(&mut ex.set_property, func);
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExSetMap(ex) => {
            on_kismet(&mut ex.map_property, func);
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExSetConst(ex) => {
            on_pointer(&mut ex.inner_property, func);
            for el in &mut ex.elements {
                on_kismet(el, func)
            }
        }
        K::ExMapConst(ex) => {
            on_pointer(&mut ex.key_property, func);
            on_pointer(&mut ex.value_property, func);
        }
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
        K::ExComputedJump(ex) => on_kismet(&mut ex.code_offset_expression, func),
        K::ExPopExecutionFlowIfNot(ex) => on_kismet(&mut ex.boolean_expression, func),
        K::ExInterfaceContext(ex) => on_kismet(&mut ex.interface_value, func),
        K::ExObjToInterfaceCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target, func);
        }
        K::ExCrossInterfaceCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target, func);
        }
        K::ExInterfaceToObjCast(ex) => {
            func(&mut ex.class_ptr);
            on_kismet(&mut ex.target, func);
        }
        K::ExAddMulticastDelegate(ex) => {
            on_kismet(&mut ex.delegate, func);
            on_kismet(&mut ex.delegate_to_add, func);
        }
        K::ExClearMulticastDelegate(ex) => on_kismet(&mut ex.delegate_to_clear, func),
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
        K::ExArrayGetByRef(ex) => {
            on_kismet(&mut ex.array_variable, func);
            on_kismet(&mut ex.array_index, func);
        }
        K::ExClassSparseDataVariable(ex) => on_pointer(&mut ex.variable, func),
        K::ExFieldPathConst(ex) => on_kismet(&mut ex.value, func),
        _ => (),
    }
}
