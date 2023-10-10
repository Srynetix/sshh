use ratatui::widgets::ListState;

#[derive(Default)]
pub struct StatefulList<T> {
    data: Vec<T>,
    state: ListState,
}

pub struct StatefulListIterator<'a, T> {
    values: &'a StatefulList<T>,
    index: usize,
}

impl<'a, T> Iterator for StatefulListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.values.data.len() {
            let result = &self.values.data[self.index];
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<T> StatefulList<T> {
    pub fn iter(&self) -> StatefulListIterator<T> {
        StatefulListIterator {
            values: self,
            index: 0,
        }
    }

    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn select_next(&mut self) {
        if self.data.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.data.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn select_first(&mut self) {
        self.state.select(Some(0));
    }

    pub fn select_previous(&mut self) {
        if self.data.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.data.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn current(&self) -> Option<&T> {
        self.state.selected().and_then(|v| self.data.get(v))
    }

    pub fn set_data(&mut self, data: Vec<T>) {
        self.data = data;
        self.state.select(None);
    }
}
