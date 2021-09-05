use automation::context::Context;

fn main() {
    let mut ctx = Context::default();
    ctx.capturer_mut().frame().save("screenshot.png").unwrap();
}
