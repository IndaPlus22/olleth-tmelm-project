use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Local};


pub struct FileInfo {
    name: String,
    datatype: String,
    path: PathBuf,
    size: usize,
    date: String,
}

impl Clone for FileInfo {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            datatype: self.datatype.clone(),
            path: self.path.clone(),
            size: self.size,
            date: self.date.clone(),
        }
    }
}

impl FileInfo {

    pub fn new(path: &Path) -> FileInfo {
        println!("{}", path.display());
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => "".to_string(),
        };
    
        let datatype = match path.extension() {
            Some(ext) => ext.to_string_lossy().into_owned(),
            None => "".to_string(),
        };
    
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => panic!("Failed to read file metadata: {}", e),
        };
    
        let size = metadata.len() as usize;
    
        let date = match metadata.modified() {
            Ok(modified_time) => {
                let dt: DateTime<Local> = modified_time.into();
                format!("{}", dt.format("%Y-%m-%d %H:%M:%S"))
            },
            Err(e) => panic!("Failed to read file modification time: {}", e),
        };

        FileInfo {
            name,
            datatype,
            path: path.to_path_buf(),
            size,
            date,        
        }
    
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn datatype(&self) -> &str {
        &self.datatype
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn date(&self) -> &str {
        &self.date
    }
}
