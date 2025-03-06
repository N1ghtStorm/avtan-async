use avtan_async::{hello_async, Executor};



fn main() {
    let executor = Executor::new();
    executor.spawn(hello_async());
    executor.run();
}
