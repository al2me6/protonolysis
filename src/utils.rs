#[derive(Clone, PartialEq, Debug)]
pub struct StoreOnNthCall<const N: usize, T> {
    set_count: usize,
    value: Option<T>,
}

impl<const N: usize, T> Default for StoreOnNthCall<N, T> {
    fn default() -> Self {
        Self {
            set_count: 0,
            value: None,
        }
    }
}

impl<const N: usize, T> StoreOnNthCall<N, T> {
    pub fn set(&mut self, value: T) {
        if self.value.is_some() {
            return;
        }
        self.set_count += 1;
        if self.set_count == N {
            self.value = Some(value);
        }
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }
}
