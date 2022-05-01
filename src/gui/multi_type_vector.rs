use druid::im::Vector;
use druid::widget::ListIter;
use druid::{Data, Lens};

use crate::clipboard::Content;

#[derive(Debug, Clone, Eq, PartialEq, Data)]
pub enum ContentType {
    All,
    Text,
    Image,
}

#[derive(Debug, Clone, Lens)]
pub struct MultiVector {
    content_type: ContentType,
    contents: Vector<Content>,
}

impl MultiVector {
    pub fn new(content_type: ContentType, contents: Vector<Content>) -> Self {
        Self {
            content_type,
            contents,
        }
    }

    pub fn push_front(&mut self, content: Content) {
        self.contents.push_front(content);
    }

    pub fn pop_back(&mut self) -> Option<Content> {
        self.contents.pop_back()
    }

    pub fn len(&self) -> usize {
        match self.content_type {
            ContentType::All => self.contents.data_len(),

            ContentType::Text => self
                .contents
                .iter()
                .filter(|content| matches!(content, Content::Text(_)))
                .count(),

            ContentType::Image => self
                .contents
                .iter()
                .filter(|content| matches!(content, Content::Image(_)))
                .count(),
        }
    }
}

impl Data for MultiVector {
    fn same(&self, other: &Self) -> bool {
        self.content_type == other.content_type && self.contents.same(&other.contents)
    }
}

impl ListIter<Content> for MultiVector {
    fn for_each(&self, mut cb: impl FnMut(&Content, usize)) {
        match self.content_type {
            ContentType::All => self.contents.for_each(cb),

            ContentType::Text => self
                .contents
                .iter()
                .filter(|content| matches!(content, Content::Text(_)))
                .enumerate()
                .for_each(|(i, content)| cb(content, i)),

            ContentType::Image => self
                .contents
                .iter()
                .filter(|content| matches!(content, Content::Image(_)))
                .enumerate()
                .for_each(|(i, content)| cb(content, i)),
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Content, usize)) {
        match self.content_type {
            ContentType::All => self.contents.for_each_mut(cb),

            ContentType::Text => self
                .contents
                .iter()
                .filter(|content| matches!(content, Content::Text(_)))
                .cloned()
                .enumerate()
                .for_each(|(i, mut content)| cb(&mut content, i)),

            ContentType::Image => self
                .contents
                .iter()
                .filter(|content| matches!(content, Content::Image(_)))
                .cloned()
                .enumerate()
                .for_each(|(i, mut content)| cb(&mut content, i)),
        }
    }

    fn data_len(&self) -> usize {
        self.len()
    }
}
