pub fn optimize_bf(mut bf: &str) -> String {
    // check if the program starts with > and ends with <
    if bf.starts_with('>') && bf.ends_with('<') {
        bf = &bf[1..bf.len() - 1];
    }

    // parse
    let ir = bfc_ir::parse(bf).unwrap();

    // optimize_bf
    let (ir, _) = bfc_ir::optimize(ir, bfc_ir::OptimisationsFlags::all());

    // decompile
    bfc_ir::decompile(&ir)
}
