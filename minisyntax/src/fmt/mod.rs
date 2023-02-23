use crate::*;

use std::fmt::Write;

use std::fmt::Error;
use std::result::Result;

mod expr;
use expr::*;

mod ty;
use ty::*;

pub fn program_to_string(prog: &Program) -> String {
    let mut wr = String::new();
    let mut fns: Vec<(_, _)> = prog.functions.iter().collect();
    let mut comptypes: Vec<Type> = Vec::new();
    fns.sort_by_key(|(k, _)| k.0);
    for (fn_name, f) in fns {
        let start = prog.start == fn_name;
        fmt_function(fn_name, f, start, &mut wr, &mut comptypes).unwrap();
    }

    let mut out = String::new();

    let mut i = 0;
    while i < comptypes.len() {
        let c = comptypes[i];
        out.push_str(&fmt_comptype(i, c, &mut comptypes));
        i += 1;
    }
    out.push_str(&wr);
    out
}

pub fn dump_program(prog: &Program) {
    println!("{}", program_to_string(prog));
}

fn fmt_function(fn_name: FnName, f: Function, start: bool, wr: &mut String, comptypes: &mut Vec<Type>) -> Result<(), Error> {
    let start_str = if start {
        "[start] "
    } else { "" };
    let fn_name = fn_name_to_string(fn_name);
    let args: Vec<_> = f.args.iter().map(|(x, _)| {
            let ident = local_name_to_string(x);
            let ty = type_to_string(f.locals.index_at(x).ty, comptypes);

            format!("{ident}: {ty}")
        }).collect();
    let args = args.join(", ");

    let mut ret_ty = String::from("!!!");
    if let Some((ret, _)) = f.ret {
        ret_ty = type_to_string(f.locals.index_at(ret).ty, comptypes);
    }
    writeln!(wr, "{start_str}fn {fn_name}({args}) -> {ret_ty} {{")?;

    // fmt locals
    let mut locals: Vec<_> = f.locals.keys().collect();
    locals.sort_by_key(|l| l.0.get());
    for l in locals {
        let ty = f.locals.index_at(l).ty;
        writeln!(wr, "  let {}: {};", local_name_to_string(l), type_to_string(ty, comptypes))?;
    }

    let mut blocks: Vec<(_, _)> = f.blocks.iter().collect();
    blocks.sort_by_key(|(k, _)| k.0);
    for (bb_name, bb) in blocks {
        let start = f.start == bb_name;
        fmt_bb(bb_name, bb, start, wr, comptypes)?;
    }
    writeln!(wr, "}}")?;
    writeln!(wr, "")?;

    Ok(())
}

fn fmt_bb(bb_name: BbName, bb: BasicBlock, start: bool, wr: &mut String, comptypes: &mut Vec<Type>) -> Result<(), Error> {
    if start {
        writeln!(wr, "  bb{} [start]:", bb_name.0.get())?;
    } else {
        writeln!(wr, "  bb{}:", bb_name.0.get())?;
    }

    for st in bb.statements.iter() {
        fmt_statement(st, wr, comptypes)?;
    }
    fmt_terminator(bb.terminator, wr, comptypes)?;

    Ok(())
}

fn fmt_statement(st: Statement, wr: &mut String, comptypes: &mut Vec<Type>) -> Result<(), Error> {
    match st {
        Statement::Assign { destination, source } => {
            writeln!(wr, "    {} = {};", place_expr_to_string(destination, comptypes), value_expr_to_string(source, comptypes))?
        },
        Statement::Finalize { place, fn_entry } => {
            writeln!(wr, "    Finalize({}, {});", place_expr_to_string(place, comptypes), fn_entry)?
        },
        Statement::StorageLive(local) => {
            writeln!(wr, "    StorageLive({});", local_name_to_string(local))?
        },
        Statement::StorageDead(local) => {
            writeln!(wr, "    StorageDead({});", local_name_to_string(local))?
        },
    }

    Ok(())
}

fn fmt_call(callee: &str, arguments: List<ValueExpr>, ret: Option<PlaceExpr>, next_block: Option<BbName>, wr: &mut String, comptypes: &mut Vec<Type>) -> Result<(), Error> {
    let args: Vec<_> = arguments.iter().map(|x| value_expr_to_string(x, comptypes)).collect();
    let args = args.join(", ");

    let mut r = String::from("!!!");
    if let Some(ret) = ret {
        r = place_expr_to_string(ret, comptypes);
    }
    let mut next = String::new();
    if let Some(next_block) = next_block {
        next = format!(" -> {}", bb_name_to_string(next_block));
    }
    writeln!(wr, "    {r} = {callee}({args}){next};")?;

    Ok(())
}

fn fmt_terminator(t: Terminator, wr: &mut String, comptypes: &mut Vec<Type>) -> Result<(), Error> {
    match t {
        Terminator::Goto(bb) => {
            writeln!(wr, "    goto -> {};", bb_name_to_string(bb))?;
        },
        Terminator::If {
            condition,
            then_block,
            else_block,
        } => {
            writeln!(wr, "    if {} {{", value_expr_to_string(condition, comptypes))?;
            writeln!(wr, "      goto -> {};", bb_name_to_string(then_block))?;
            writeln!(wr, "    }} else {{")?;
            writeln!(wr, "      goto -> {};", bb_name_to_string(else_block))?;
            writeln!(wr, "    }}")?;
        },
        Terminator::Unreachable => {
            writeln!(wr, "    unreachable;")?;
        }
        Terminator::Call {
            callee,
            arguments,
            ret,
            next_block,
        } => {
            let callee = fn_name_to_string(callee);
            let arguments = arguments.iter().map(|(x, _)| x).collect();
            let ret = ret.map(|(x, _)| x);
            fmt_call(&callee, arguments, ret, next_block, wr, comptypes)?;
        },
        Terminator::Return => {
            writeln!(wr, "    return;")?;
        },
        Terminator::CallIntrinsic {
            intrinsic,
            arguments,
            ret,
            next_block,
        } => {
            let callee = match intrinsic {
                Intrinsic::Exit => "exit",
                Intrinsic::PrintStdout => "print",
                Intrinsic::PrintStderr => "eprint",
                Intrinsic::Allocate => "allocate",
                Intrinsic::Deallocate => "deallocate",
            };
            fmt_call(callee, arguments, ret, next_block, wr, comptypes)?;
        },
    }

    Ok(())
}