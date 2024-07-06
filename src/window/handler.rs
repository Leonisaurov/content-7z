use crate::window::window::Window;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;

pub enum HandleSituatonType {
    SUCESS(bool),
    DENIED,
    KEY(char),
}

pub trait Handler {
    fn execute(&self, win: &mut Window, hand_type: HandleSituatonType);
}

type NormalHand<T> = fn(&mut Window, HandleSituatonType, RefMut<T>);

pub struct NormalHandler<T> {
    job: NormalHand<T>,
    data: Rc<RefCell<T>>,
}

impl<T> NormalHandler<T> {
    pub fn new(job: NormalHand<T>, data: T) -> Self {
        Self {
            job,
            data: Rc::new(
                RefCell::new(data)
            ),
        }
    }
}

impl<T> Handler for NormalHandler<T> {
    fn execute(&self, win: &mut Window, hand_type: HandleSituatonType) {
        (self.job)(win, hand_type, self.data.borrow_mut())
    }
}
