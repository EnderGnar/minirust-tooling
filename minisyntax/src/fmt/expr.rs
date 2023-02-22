use super::*;

pub fn place_expr_to_string(p: PlaceExpr) -> String {
    match p {
        PlaceExpr::Local(l) => local_name_to_string(l),
        PlaceExpr::Deref { operand, .. } => {
            format!("*{}", value_expr_to_string(operand.get()))
        },
        PlaceExpr::Field { root, field } => {
            let root = root.get();
            format!("{}.{}", place_expr_to_string(root), field)
        },
        PlaceExpr::Index { root, index } => {
            let root = root.get();
            let index = index.get();
            format!("{}[{}]", place_expr_to_string(root), value_expr_to_string(index))
        },
    }
}

pub fn local_name_to_string(l: LocalName) -> String {
    format!("_{}", l.0.get())
}

fn constant_to_string(c: Constant) -> String {
    match c {
        Constant::Int(int) => int.to_string(),
        Constant::Bool(b) => b.to_string(),
        Constant::Variant { .. } => panic!("enums are unsupported!"),
    }
}

pub fn value_expr_to_string(v: ValueExpr) -> String {
    match v {
        ValueExpr::Constant(c, _ty) => constant_to_string(c),
        ValueExpr::Tuple(l, t) => {
            let (lparen, rparen) = match t {
                Type::Array { .. } => ('[', ']'),
                Type::Tuple { .. } => ('(', ')'),
                _ => panic!(),
            };
            let l: Vec<_> = l.iter().map(value_expr_to_string).collect();
            let l = l.join(", ");

            format!("{lparen}{l}{rparen}")
        },
        ValueExpr::Union { field, expr, union_ty } => {
            let union_ty = type_to_string(union_ty);
            let expr = value_expr_to_string(expr.get());
            format!("{union_ty} {{ f{field} : {expr} }}")
        },
        ValueExpr::Load { destructive, source } => {
            let source = source.get();
            let source = place_expr_to_string(source);
            let load_name = match destructive {
                true => "move",
                false => "load",
            };
            format!("{load_name}({source})")
        },
        ValueExpr::AddrOf { target, ptr_ty: PtrType::Raw { .. } } => {
            let target = target.get();
            let target = place_expr_to_string(target);
            format!("&raw {target}")
        },
        ValueExpr::AddrOf { target, ptr_ty: PtrType::Ref { mutbl, .. } } => {
            let target = target.get();
            let target = place_expr_to_string(target);
            let mutbl = match mutbl {
                Mutability::Mutable => "mut ",
                Mutability::Immutable => "",
            };
            format!("&{mutbl}{target}")
        },
        ValueExpr::AddrOf { target: _, ptr_ty: _ } => {
            panic!("unsupported ptr_ty for AddrOr!")
        },
        ValueExpr::UnOp { operator, operand } => {
            let operand = value_expr_to_string(operand.get());
            match operator {
                UnOp::Int(UnOpInt::Neg, _int_ty) => format!("-{}", operand),
                UnOp::Int(UnOpInt::Cast, _int_ty) => format!("{} as _", operand),
                UnOp::Ptr2Ptr(_ptr_ty) => format!("{} as _", operand),
                UnOp::Ptr2Int => format!("{} as _", operand),
                UnOp::Int2Ptr(_ptr_ty) => format!("{} as _", operand),
            }
        },
        ValueExpr::BinOp { operator: BinOp::Int(int_op, int_ty), left, right } => {
            let int_op = match int_op {
                BinOpInt::Add => '+',
                BinOpInt::Sub => '-',
                BinOpInt::Mul => '*',
                BinOpInt::Div => '/',
                BinOpInt::Rem => '%',
            };

            let int_ty = int_type_to_string(int_ty);
            let int_op = format!("{int_op}_{int_ty}");

            let l = value_expr_to_string(left.get());
            let r = value_expr_to_string(right.get());

            format!("{l} {int_op} {r}")
        },
        ValueExpr::BinOp { operator: BinOp::IntRel(rel), left, right } => {
            let rel = match rel {
                IntRel::Lt => "<",
                IntRel::Le => "<=",
                IntRel::Gt => ">",
                IntRel::Ge => ">=",
                IntRel::Eq => "==",
                IntRel::Ne => "!=",
            };

            let l = value_expr_to_string(left.get());
            let r = value_expr_to_string(right.get());

            format!("{l} {rel} {r}")
        },
        ValueExpr::BinOp { operator: BinOp::PtrOffset { inbounds }, left, right } => {
            let offset_name = match inbounds {
                true => "offset_inbounds",
                false => "offset_wrapping",
            };
            let l = value_expr_to_string(left.get());
            let r = value_expr_to_string(right.get());
            format!("{offset_name}({l}, {r})")
        }
    }
}

fn int_type_to_string(int_ty: IntType) -> String {
    let signed = match int_ty.signed {
        Signed => "i",
        Unsigned => "u",
    };
    let bits = int_ty.size.bits();

    format!("{signed}{bits}")
}

pub fn type_to_string(t: Type) -> String {
    match t {
        Type::Int(int_ty) => int_type_to_string(int_ty),
        Type::Bool => String::from("bool"),
        Type::Ptr(PtrType::Ref { mutbl: Mutability::Mutable, .. }) => String::from("&mut _"),
        Type::Ptr(PtrType::Ref { mutbl: Mutability::Immutable, .. }) => String::from("&_"),
        Type::Ptr(PtrType::Box { .. }) => String::from("Box<_>"),
        Type::Ptr(PtrType::Raw { .. }) => String::from("*_"),
        Type::Tuple { fields, .. } => {
            let fields: Vec<_> = fields.iter().map(|(_, ty)| type_to_string(ty)).collect();
            let fields = fields.join(", ");

            format!("({fields})")
        },
        Type::Array { elem, count } => {
            let elem = type_to_string(elem.get());
            format!("[{}; {}]", elem, count)
        },
        Type::Union { .. } => format!("{:?}", t),
        Type::Enum { .. } => panic!("enums are unsupported!"),
    }
}

pub fn bb_name_to_string(bb: BbName) -> String {
    format!("bb{}", bb.0.get())
}

pub fn fn_name_to_string(fn_name: FnName) -> String {
    format!("f{}", fn_name.0.get())
}
