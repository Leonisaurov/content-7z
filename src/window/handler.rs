use crate::window::window::Window;
pub enum HandleSituatonType {
    SUCESS,
    DENIED
}

pub trait Handler {
    fn execute(&self, win: &mut Window, hand_type: HandleSituatonType);
}

type Hand<T> = fn(&mut Window, HandleSituatonType, &Vec<T>);

pub struct NormalHandler<T> {
    job: Hand<T>,
    data: Vec<T>
}

impl<T> NormalHandler<T> {
    pub fn new(job: Hand<T>, data: Vec<T>) -> Self {
        Self {
            job,
            data
        }
    }
}

impl<T> Handler for NormalHandler<T> {
    fn execute(&self, win: &mut Window, hand_type: HandleSituatonType) {
        (self.job)(win, hand_type, &self.data)
    }
}
