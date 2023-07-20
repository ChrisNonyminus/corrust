use super::{Corruptor, Input};

pub struct CorruptorChain {
    pub og_data: Input,
    pub data: Input,
}

impl CorruptorChain {
    pub fn corrupt(&mut self, corruptor: &dyn Corruptor) -> &mut CorruptorChain {
        self.data.data = corruptor.corrupt(&self.data);
        self
    }
}
