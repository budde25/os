pub mod cpu;

use cpu::Cpu;
use staticvec::StaticVec;

static CPUS: StaticVec<Cpu, 8> = StaticVec::new();
