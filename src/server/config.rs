use crate::TrentFile;

pub struct ServerData {
    files: Vec<TrentFile>,
    file_information: &'static [u8],
}

impl ServerData {
    pub fn from_files(files: Vec<TrentFile>) -> Self {
        let mut size = 0;
        for file in &files {
            size += file.name().len()
                + 1 // null byte
                + 4; // u32 file information flags
        }

        let mut file_information = Vec::<u8>::with_capacity(size);

        for file in &files {
            file_information.extend_from_slice(file.name().as_bytes());
            file_information.push(0b0);
            file_information.extend_from_slice(&file.flags().to_be_bytes());
        }

        Self {
            files,
            file_information: file_information.leak(),
        }
    }

    pub fn get_files(&'static self) -> &'static [TrentFile] {
        &self.files
    }

    pub fn file_information(&self) -> &[u8] {
        &self.file_information
    }
}
