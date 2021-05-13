use lazy_static::lazy_static;

mod handlers;
mod idt;

lazy_static! {
    static ref IDT: idt::IDT = {
        let mut idt = idt::IDT::new();
        idt.set_handler(0, handlers::divide_by_zero);
        // idt.set_handler(14, handlers::page_fault_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}
