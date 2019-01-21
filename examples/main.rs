use parapet;

fn main() {
    let manager = parapet::Manager::new()
        .expect("failed to init manager");
    for screen in manager.screens() {
        screen.set()
    }
}
