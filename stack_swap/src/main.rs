use core::arch::asm;

const STACK_SIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: usize,
}

fn foo() -> !
{
    println!("I LOVE WAKING UP ON A NEW STACK!");
    #[allow(clippy::empty_loop)]
    loop {}
}

unsafe fn gt_switch(new: *const ThreadContext) {
    asm!(
        "mov rsp, [{0}]",
        "ret",
        in(reg) new,
    );
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = [0u8; STACK_SIZE as usize];

    unsafe {
        let stack_btm = stack.as_mut_ptr().offset(STACK_SIZE);
        let sb_align = (stack_btm as usize & !15) as *mut u8;
        std::ptr::write(sb_align.offset(-16) as *mut usize, foo as usize);
        ctx.rsp = sb_align.offset(-16) as usize;
        for i in 0..STACK_SIZE {
            println!("{}: {}", sb_align.offset(-i) as usize, *sb_align.offset(-i));
        }
        gt_switch(&ctx);
    }
}
