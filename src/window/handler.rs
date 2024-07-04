use crate::window::window::Window;
pub enum HandleSituatonType {
    SUCESS,
    DENIED
}

pub type Handler = fn(&mut Window, HandleSituatonType);
