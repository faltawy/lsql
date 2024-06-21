use chrono::Utc;
use comfy_table::Table;

#[derive(Debug, Copy, Clone)]
pub enum FileType {
    Directory,
    File,
    Other,
}

#[derive(Debug, Copy, Clone)]
pub enum FilePermission {
    Read,
    Write,
    Execute,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: u64,
    pub modified: chrono::DateTime<Utc>,
    pub name: String,
    pub file_type: FileType,
    pub path: String,
}


impl FileInfo {
    pub fn human_readable_size(&self) -> String {
        let size = self.size;
        let kb = 1024;
        let mb = kb * 1024;
        let gb = mb * 1024;
        let tb = gb * 1024;
        if size < kb {
            format!("{} B", size)
        } else if size < mb {
            format!("{:.2} KB", size as f64 / kb as f64)
        } else if size < gb {
            format!("{:.2} MB", size as f64 / mb as f64)
        } else if size < tb {
            format!("{:.2} GB", size as f64 / gb as f64)
        } else {
            format!("{:.2} TB", size as f64 / tb as f64)
        }
    }

    pub fn human_readable_modified(&self) -> String {
        self.modified.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[derive(Debug)]
pub struct FileQuerySet {
    result: Vec<FileInfo>,
}

impl FileQuerySet {
    pub fn new(files: Vec<FileInfo>) -> Self {
        FileQuerySet { result: files }
    }

    pub fn table_them(&self) -> Table{
        let mut table = Table::new();
        table
        .set_header(vec![
            "Name",
            "Size",
            "Modified",
        ]);
        for file in &self.result {
            table.add_row(vec![
                file.name.clone(),
                file.human_readable_size(),
                file.human_readable_modified(),
            ]);
        };
        return table;
    }
}
