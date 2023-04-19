use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Local};

///A Struct that holds a files metadata.
pub struct FileInfo {
    name: String,
    datatype: String,
    path: PathBuf,
    size: usize,
    date: String,
}

impl Clone for FileInfo {
    ///Clones the FileInfo
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
    /// Constructs a new `FileInfo`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to a file.
    pub fn new(path: &Path) -> FileInfo {
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
    
    ///returns the name from the FileInfo.
    pub fn name(&self) -> &str {
        &self.name
    }

    ///returns the datatype from the FileInfo.
    pub fn datatype(&self) -> &str {
        &self.datatype
    }

    ///returns the path from the FileInfo.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    ///returns the size from the FileInfo.
    pub fn size(&self) -> usize {
        self.size
    }

    ///returns the date from the FileInfo.
    pub fn date(&self) -> &str {
        &self.date
    }
}
