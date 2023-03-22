use crate::build::*;

pub fn layout(size: Size, align: Align) -> Layout {
    Layout {
        size,
        align,
        inhabited: true, // currently everything is inhabited (enums don't exist yet).
    }
}

pub fn int_ty(signed: Signedness, size: Size) -> Type {
    Type::Int(IntType { signed, size })
}

pub fn bool_ty() -> Type {
    Type::Bool
}

pub fn ref_ty(pointee: Layout) -> Type {
    Type::Ptr(PtrType::Ref {
        mutbl: Mutability::Immutable,
        pointee,
    })
}

pub fn ref_mut_ty(pointee: Layout) -> Type {
    Type::Ptr(PtrType::Ref {
        mutbl: Mutability::Mutable,
        pointee,
    })
}

pub fn box_ty(pointee: Layout) -> Type {
    Type::Ptr(PtrType::Box { pointee })
}

pub fn raw_ptr_ty(pointee: Layout) -> Type {
    Type::Ptr(PtrType::Raw { pointee })
}

pub fn tuple_ty(f: &[(Size, Type)], size: Size) -> Type {
    Type::Tuple {
        fields: f.iter().copied().collect(),
        size,
    }
}

pub fn union_ty(f: &[(Size, Type)], size: Size) -> Type {
    let chunks = list![(Size::ZERO, size)];
    Type::Union {
        fields: f.iter().copied().collect(),
        size,
        chunks,
    }
}

pub fn array_ty(elem: Type, count: impl Into<Int>) -> Type {
    Type::Array {
        elem: GcCow::new(elem),
        count: count.into(),
    }
}

pub fn ptype(ty: Type, align: Align) -> PlaceType {
    PlaceType { ty, align }
}
