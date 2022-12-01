use crate::*;

pub fn translate_bb<'tcx>(bb: &rs::BasicBlockData<'tcx>, fcx: &mut FnCtxt<'tcx>) -> mini::BasicBlock {
    let mut statements = specr::List::new();
    for stmt in bb.statements.iter() {
        translate_stmt(stmt, fcx, &mut statements);
    }
    mini::BasicBlock {
        statements,
        terminator: translate_terminator(bb.terminator(), fcx),
    }
}

fn translate_stmt<'tcx>(stmt: &rs::Statement<'tcx>, fcx: &mut FnCtxt<'tcx>, statements: &mut specr::List<mini::Statement>) {
    match &stmt.kind {
        rs::StatementKind::Assign(box (place, rval)) => {
            statements.push(
                mini::Statement::Assign {
                    destination: translate_place(place, fcx),
                    source: translate_rvalue(rval, fcx),
                }
            );
        },
        rs::StatementKind::StorageLive(local) => {
            statements.push(
                mini::Statement::StorageLive(fcx.localname_map[&local])
            );
        },
        rs::StatementKind::StorageDead(local) => {
            statements.push(
                mini::Statement::StorageDead(fcx.localname_map[&local])
            );
        },
        rs::StatementKind::Deinit(..) => { /* this has no mini::_ equivalent. */ },
        rs::StatementKind::Retag(..) => { /* this has no mini::_ equivalent. */ },
        x => {
            dbg!(x);
            todo!()
        }
    }
}

fn translate_terminator<'tcx>(terminator: &rs::Terminator<'tcx>, fcx: &mut FnCtxt<'tcx>) -> mini::Terminator {
    match &terminator.kind {
        rs::TerminatorKind::Return => mini::Terminator::Return,
        rs::TerminatorKind::Goto { target } => mini::Terminator::Goto(fcx.bbname_map[&target]),
        rs::TerminatorKind::Call { func, target, destination, args, .. } => {
            let rs::Operand::Constant(box f) = func else { panic!() };
            let rs::ConstantKind::Val(_, f) = f.literal else { panic!() };
            let rs::TyKind::FnDef(f, substs_ref) = f.kind() else { panic!() };
            let key = (*f, *substs_ref);
            if !fcx.fnname_map.contains_key(&key) {
                let fname = fcx.fnname_map.len();
                let fname = mini::FnName(specr::Name(fname as _));
                fcx.fnname_map.insert(key, fname);
            }
            mini::Terminator::Call {
                callee: fcx.fnname_map[&key],
                arguments: args.iter().map(|x| (translate_operand(x, fcx), arg_abi())).collect(),
                ret: (translate_place(&destination, fcx), arg_abi()),
                next_block: fcx.bbname_map[&target.unwrap()], // TODO handle `None`: it means that the call necessarily diverges, see the docs.
            }
        }
        // TODO Assert is unsupported!
        rs::TerminatorKind::Assert { target, .. } => {
            mini::Terminator::Goto(fcx.bbname_map[&target])
        }
        x => {
            dbg!(x);
            todo!()
        }
    }
}
