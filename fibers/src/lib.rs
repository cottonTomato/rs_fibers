#![feature(naked_functions)]
use std::{arch::asm, ptr};

const DEFAULT_STACK_SIZE: usize = 2 * 1024 * 1024;
const MAX_THREADS: usize = 4;
static mut RUNTIME: *mut Runtime = ptr::null_mut();

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: usize,
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbx: usize,
    rbp: usize,
}

#[derive(PartialEq, Eq, Debug)]
enum ThreadState {
    Available,
    Running,
    Ready,
}

struct Thread {
    stack: Box<[u8]>,
    ctx: ThreadContext,
    state: ThreadState,
}

impl Thread {
    fn new() -> Self {
        Self {
            stack: vec![0u8; DEFAULT_STACK_SIZE].into_boxed_slice(),
            ctx: ThreadContext::default(),
            state: ThreadState::Available,
        }
    }
}

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

impl Runtime {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let base_thread = Thread {
            stack: vec![0u8; DEFAULT_STACK_SIZE].into_boxed_slice(),
            ctx: ThreadContext::default(),
            state: ThreadState::Running,
        };

        let mut threads = vec![base_thread];
        let mut available_threads = (0..MAX_THREADS)
            .map(|_| Thread::new())
            .collect::<Vec<Thread>>();
        threads.append(&mut available_threads);

        Self {
            threads,
            current: 0,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            RUNTIME = self as *mut Runtime;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yeild() {}
        std::process::exit(0);
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = ThreadState::Available;
            self.t_yeild();
        }
    }

    #[inline(never)]
    fn t_yeild(&mut self) -> bool {
        let mut pos = self.current;
        while self.threads[pos].state != ThreadState::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != ThreadState::Available {
            self.threads[self.current].state = ThreadState::Ready;
        }

        self.threads[pos].state = ThreadState::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old_ctx = &mut self.threads[old_pos].ctx as *mut ThreadContext;
            let new_ctx = &self.threads[pos].ctx as *const ThreadContext;

            #[cfg(not(target_os = "macos"))]
            asm!(
                "call switch",
                in("rdi") old_ctx,
                in("rsi") new_ctx,
                clobber_abi("C")
            );

            #[cfg(target_os = "macos")]
            asm!(
                "call _switch",
                in("rdi") old_ctx,
                in("rsi") new_ctx,
                clobber_abi("C")
            );
        }

        // Execution never reches here
        !self.threads.is_empty()
    }

    pub fn spawn(&mut self, f: fn()) {
        let available_thread = self
            .threads
            .iter_mut()
            .find(|t| t.state == ThreadState::Available)
            .expect("No Threads Available");
        let stack_size = available_thread.stack.len();

        unsafe {
            let s_ptr = available_thread.stack.as_mut_ptr().add(stack_size);
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            ptr::write(s_ptr.offset(-16) as *mut usize, guard as usize);
            ptr::write(s_ptr.offset(-24) as *mut usize, skip as usize);
            ptr::write(s_ptr.offset(-32) as *mut usize, f as usize);

            available_thread.ctx.rsp = s_ptr.offset(-32) as usize;
        }

        available_thread.state = ThreadState::Ready;
    }
}

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME;
        (*rt_ptr).t_return();
    }
}

#[naked]
unsafe extern "C" fn skip() {
    asm!("ret", options(noreturn));
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME;
        (*rt_ptr).t_yeild();
    }
}

#[naked]
#[no_mangle]
unsafe extern "C" fn switch() {
    asm!(
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "ret",
        options(noreturn)
    );
}
