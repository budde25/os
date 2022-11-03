enum State {
    Unused,
    Embryo,
    Sleeping,
    Runable,
    Running,
    Zombie,
}

pub(crate) struct Process {
    mem_size: usize,
    // pgdir
    // kstack
    state: State,
    pid: usize,
    killed: bool,
}
