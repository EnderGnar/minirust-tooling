use crate::*;

mod build;
use build::*;

pub fn assert_ub(prog: Program, msg: &str) {
    assert_eq!(run_program(prog), Outcome::Ub(msg.to_string()));
}

pub fn assert_stop(prog: Program) {
    assert_eq!(run_program(prog), Outcome::Stop);
}

pub fn assert_unwell(prog: Program) {
    assert_eq!(run_program(prog), Outcome::Unwell);
}

#[test]
fn too_large_alloc() {
    fn program_alloc(bytes: Int) -> Program {
        let count = bytes;
        let ty = array_ty(bool_ty(), count);

        let locals = vec![ptype(ty, align(1))];
        let stmts = vec![live(0), dead(0)];
        small_program(&locals, &stmts)
    }

    run_sequential(|| {
        let large = Int::from(2).pow(BasicMemory::PTR_SIZE.bits());
        assert_unwell(program_alloc(large));

        let small = Int::from(2);
        assert_stop(program_alloc(small));
    });
}

#[test]
fn double_live() {
    run_sequential(|| {
        let locals = vec![ <bool>::get_ptype() ];
        let stmts = vec![live(0), live(0)];
        let p = small_program(&locals, &stmts);
        assert_unwell(p);
    });
}

#[test]
fn dead_before_live() {
    run_sequential(|| {
        let locals = vec![ <bool>::get_ptype() ];
        let stmts = vec![dead(0)];
        let p = small_program(&locals, &stmts);
        assert_unwell(p);
    });
}

#[test]
fn uninit_read() {
    run_sequential(|| {
        let locals = vec![ <bool>::get_ptype(); 2];
        let stmts = vec![
            live(0),
            live(1),
            assign(
                local(0),
                load(local(1)),
            ),
        ];
        let p = small_program(&locals, &stmts);
        assert_ub(p, "load at type PlaceType { ty: Bool, align: Align { raw: Small(1) } } but the data in memory violates the validity invariant");
    });
}

// see https://github.com/rust-lang/miri/issues/845
#[test]
fn no_preserve_padding() {
    // type Pair = (u8, u16);
    // union Union { f0: Pair, f1: u32 }
    //
    // let _0: Union;
    // let _1: Pair;
    // let _2: *const u8;
    // let _3: u8;
    //
    // _0.f1 = 0;
    // _1 = _0.f0;
    // _2 = &raw _1;
    // _2 = load(_2).offset(1)
    // _3 = *_2;

    run_sequential(|| {
        let pair_ty = tuple_ty(&[
                (size(0), u8::get_type()),
                (size(2), u16::get_type())
            ], size(4));
        let pair_pty = ptype(pair_ty, align(2));

        let union_ty = union_ty(&[
                (size(0), pair_ty),
                (size(0), u32::get_type()),
            ], size(4));
        let union_pty = ptype(union_ty, align(4));

        let locals = vec![
            union_pty,
            pair_pty,
            <*const u8>::get_ptype(),
            <u8>::get_ptype(),
        ];

        let stmts = vec![
            live(0),
            live(1),
            live(2),
            live(3),
            assign(
                field(local(0), 1),
                const_int::<u32>(0)
            ),
            assign(
                local(1),
                load(field(local(0), 0))
            ),
            assign(
                local(2),
                addr_of(
                    local(1),
                    <*const u8>::get_type(),
                ),
            ),
            assign(
                local(2),
                ptr_offset(
                    load(local(2)),
                    const_int::<u32>(1),
                    true,
                )
            ),
            assign(
                local(3),
                load(deref(load(local(2)), <u8>::get_ptype())),
            ),
        ];

        let p = small_program(&locals, &stmts);
        dump_program(&p);
        assert_ub(p, "load at type PlaceType { ty: Int(IntType { signed: Unsigned, size: Size { raw: Small(1) } }), align: Align { raw: Small(1) } } but the data in memory violates the validity invariant");
    });
}
