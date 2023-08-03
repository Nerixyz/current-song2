use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

pub struct Image {
    pub content_type: String,
    pub data: Vec<u8>,
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("content_type", &self.content_type)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct ImageStore {
    images: HashMap<usize, (usize, Option<Image>)>,
    next_id: usize,
}

impl ImageStore {
    pub fn new() -> Self {
        Self {
            images: HashMap::default(),
            next_id: 0,
        }
    }

    pub fn create_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id = self.next_id.overflowing_add(1).0;

        id
    }

    pub fn get(&self, id: usize, target_epoch: usize) -> Option<&Image> {
        self.images
            .get(&id)
            .filter(|i| i.0 == target_epoch)
            .and_then(|(_, i)| i.as_ref())
    }

    pub fn store(&mut self, slot: usize, content_type: String, data: Vec<u8>) -> usize {
        if let Some(img) = self.images.get_mut(&slot) {
            let epoch = img.0.overflowing_add(1).0;
            *img = (epoch, Some(Image { content_type, data }));
            epoch
        } else {
            self.images
                .insert(slot, (0, Some(Image { content_type, data })));
            0
        }
    }

    pub fn clear(&mut self, slot: usize) {
        if let Some(img) = self.images.get_mut(&slot) {
            img.1 = None;
        }
    }
    pub fn remove(&mut self, slot: usize) {
        self.images.remove(&slot);
    }
}
