use staticvec::StaticString;

enum State {
    UNUSED,
    EMBRYO,
    SLEEPING,
    RUNNABLE,
    RUNNING,
    ZOMBIE,
}

struct Process {
    mem_size: usize,
    // pgdir
    // kstack
    state: State,
    pid: usize,
    killed: bool,
    name: StaticString<16>,
}
