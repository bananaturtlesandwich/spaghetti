use unreal_asset::{fproperty::*, types::PackageIndex};
use FProperty as F;

pub fn on_fprop(prop: &mut FProperty, func: &mut impl FnMut(&mut PackageIndex)) {
    match prop {
        F::FEnumProperty(f) => {
            func(&mut f.enum_value);
            on_fprop(&mut f.underlying_prop, func)
        }
        F::FArrayProperty(f) => on_fprop(&mut f.inner, func),
        F::FSetProperty(f) => on_fprop(&mut f.element_prop, func),
        F::FObjectProperty(f) => func(&mut f.property_class),
        F::FSoftObjectProperty(f) => func(&mut f.property_class),
        F::FClassProperty(f) => {
            func(&mut f.property_class);
            func(&mut f.meta_class);
        }
        F::FSoftClassProperty(f) => {
            func(&mut f.property_class);
            func(&mut f.meta_class);
        }
        F::FDelegateProperty(f) => func(&mut f.signature_function),
        F::FMulticastDelegateProperty(f) => func(&mut f.signature_function),
        F::FMulticastInlineDelegateProperty(f) => func(&mut f.signature_function),
        F::FInterfaceProperty(f) => func(&mut f.interface_class),
        F::FMapProperty(f) => {
            on_fprop(&mut f.key_prop, func);
            on_fprop(&mut f.value_prop, func);
        }
        F::FByteProperty(f) => func(&mut f.enum_value),
        F::FStructProperty(f) => func(&mut f.struct_value),
        _ => (),
    }
}
