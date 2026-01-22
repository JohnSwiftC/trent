use crate::TrentFile;

pub struct ServerData {
    files: &'static [TrentFile],
    file_information: Vec<u8>,
}

impl ServerData {
    pub fn from_files(files: &'static [TrentFile]) -> Self {
        let mut size = 0;
        for file in files {
            size += file.name().len()
                + 1 // null byte
                + 4; // u32 file information flags
        }

        let mut file_information = Vec::<u8>::with_capacity(size);

        Self {
            files,
            file_information: Vec::new(),
        }
    }

    pub fn get_files(&self) -> &'static [TrentFile] {
        &self.files
    }

    pub fn file_information(&self) -> &[u8] {
        &self.file_information
    }
}
