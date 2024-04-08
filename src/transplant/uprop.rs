use unreal_asset::{types::PackageIndex, uproperty::*};
use UProperty as U;

fn on_gen(prop: &mut UGenericProperty, func: &mut impl FnMut(&mut PackageIndex)) {
    if let Some(next) = prop.u_field.next.as_mut() {
        func(next)
    }
}

pub fn on_uprop(prop: &mut UProperty, func: &mut impl FnMut(&mut PackageIndex)) {
    match prop {
        U::UGenericProperty(u) => on_gen(u, func),
        U::UEnumProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.value);
            func(&mut u.underlying_prop);
        }
        U::UArrayProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.inner);
        }
        U::USetProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.element_prop);
        }
        U::UObjectProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.property_class);
        }
        U::USoftObjectProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.property_class);
        }
        U::ULazyObjectProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.property_class);
        }
        U::UClassProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.property_class);
            func(&mut u.meta_class);
        }
        U::USoftClassProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.property_class);
            func(&mut u.meta_class);
        }
        U::UDelegateProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.signature_function);
        }
        U::UMulticastDelegateProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.signature_function);
        }
        U::UMulticastInlineDelegateProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.signature_function);
        }
        U::UInterfaceProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.interface_class);
        }
        U::UMapProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.key_prop);
            func(&mut u.value_prop);
        }
        U::UBoolProperty(u) => on_gen(&mut u.generic_property, func),
        U::UByteProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.enum_value);
        }
        U::UStructProperty(u) => {
            on_gen(&mut u.generic_property, func);
            func(&mut u.struct_value);
        }
        U::UDoubleProperty(u) => on_gen(&mut u.generic_property, func),
        U::UFloatProperty(u) => on_gen(&mut u.generic_property, func),
        U::UIntProperty(u) => on_gen(&mut u.generic_property, func),
        U::UInt8Property(u) => on_gen(&mut u.generic_property, func),
        U::UInt16Property(u) => on_gen(&mut u.generic_property, func),
        U::UInt64Property(u) => on_gen(&mut u.generic_property, func),
        U::UUInt8Property(u) => on_gen(&mut u.generic_property, func),
        U::UUInt16Property(u) => on_gen(&mut u.generic_property, func),
        U::UUInt64Property(u) => on_gen(&mut u.generic_property, func),
        U::UNameProperty(u) => on_gen(&mut u.generic_property, func),
        U::UStrProperty(u) => on_gen(&mut u.generic_property, func),
    }
}
