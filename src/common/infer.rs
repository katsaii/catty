use std::fs;
use std::path;
use std::collections::HashMap;

use log;

#[derive(Debug)]
pub struct Database {
    lookup : HashMap<path::PathBuf, usize>,
    collections : Vec<Collection>,
    files : Vec<File>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            lookup : HashMap::new(),
            collections : Vec::new(),
            files : Vec::new(),
        }
    }

    fn add_collection_canon(&mut self, path : &path::Path) -> Option<&mut Collection> {
        if !path.has_root() {
            return None;
        }
        let path_buf = path.to_path_buf();
        if let Some(cached_id) = self.lookup.get(&path_buf) {
            return Some(&mut self.collections[*cached_id]);
        }
        let parent = path.parent().and_then(|x| self.add_collection_canon(x));
        let parent_id = parent.as_ref().map(|x| x.id);
        let parent_depth = parent.as_ref().map(|x| x.depth).unwrap_or(0);
        let id = self.collections.len();
        self.lookup.insert(path_buf.clone(), id);
        let collection = Collection {
            path : path_buf,
            id : id,
            id_parent : parent_id,
            depth : parent_depth + 1,
            has_files : false,
        };
        self.collections.push(collection);
        Some(&mut self.collections[id])
    }

    pub fn add_collection(&mut self, path : &path::Path) -> Option<&mut Collection> {
        match fs::canonicalize(path) {
            Ok(canon_path) => self.add_collection_canon(canon_path.as_path()),
            Err(err) => {
                log::warn!("failed to canonicalise directory: {}\n{}", path.display(), err);
                None
            },
        }
    }

    pub fn add_file_canon(&mut self, path : &path::Path) -> Option<&mut File> {
        let collection = path.parent().and_then(|x| self.add_collection_canon(x)).unwrap();
        collection.has_files = true;
        let collection_id = collection.id;
        let id = self.files.len();
        let file = File {
            path : path.to_path_buf(),
            id : id,
            id_collection : collection_id,
        };
        self.files.push(file);
        Some(&mut self.files[id])
    }

    pub fn add_file(&mut self, path : &path::Path) -> Option<&mut File> {
        match fs::canonicalize(path) {
            Ok(canon_path) => self.add_file_canon(canon_path.as_path()),
            Err(err) => {
                log::warn!("failed to canonicalise file path: {}\n{}", path.display(), err);
                None
            },
        }
    }

    pub fn complete(self) -> (Vec<Collection>, Vec<File>) {
        (self.collections, self.files)
    }
}

pub type CollectionID = usize;

#[derive(Debug)]
pub struct Collection {
    pub path : path::PathBuf,
    pub id : CollectionID,
    pub id_parent : Option<CollectionID>,
    pub depth : usize,
    pub has_files : bool,
}

pub type FileID = usize;

#[derive(Debug)]
pub struct File {
    pub path : path::PathBuf,
    pub id : FileID,
    pub id_collection : CollectionID,
}