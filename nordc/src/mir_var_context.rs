use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type MirSharedVarContext = Rc<RefCell<MirVarContext>>;

pub struct MirVarContext {
    parent: Option<MirSharedVarContext>,
    vars: Vec<HashMap<String, usize>>,
}