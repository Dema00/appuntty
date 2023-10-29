use crate::backend::node::{HRef, Node};

enum Mode {
    Input,
    Inspect,
}

enum CurrentScreen {
    EditFile(Mode),
}

struct ProgramState {
    screen: CurrentScreen,
}
