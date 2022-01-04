use core::arch::asm;

pub struct Cpu {
    apic_id: u8,
}

impl Cpu {
    pub fn current_cpu() -> Self {
        use crate::io::LAPIC;

        assert!(
            !Self::flags().contains(Rflags::IF),
            "Interrupts must be disabled"
        );

        let apic_id = unsafe { LAPIC.id() } as u8;

        Self { apic_id }
    }

    fn flags() -> Rflags {
        Rflags::read()
    }
}

bitflags::bitflags! {
    #[repr(C)]
    pub struct Rflags: u64 {
        const CF = (1 << 0); // carry flag
        const PF = (1 << 2); // parity flag
        const AF = (1 << 4); // auxiliary flag
        const ZF = (1 << 6); // zero flag
        const SF = (1 << 7); // sign flag
        const TF = (1 << 8); // trap flag
        const IF = (1 << 9); // interrupt enable flag
        const DF = (1 << 10); // direction flag
        const OF = (1 << 11); // overflow flag
        // 12 - 13 priv level
        const NT = (1 << 14); // nested task flag
        const RF = (1 << 16); // resume flag
        const VM = (1 << 17); // virtual 8086 mode flag
        const AC = (1 << 18); // alignment check
        const VIF = (1 << 19); // virtual interrupt flag
        const VIP = (1 << 20); // virtual interrupt pending
        const ID = (1 << 20); // able ot use cpuid intstruction
    }
}

impl Rflags {
    pub fn read() -> Self {
        let value: u64;
        unsafe { asm!("pushfq; pop {}", out(reg) value, options(nomem, preserves_flags)) };
        Self::from_bits_truncate(value)
    }
}
