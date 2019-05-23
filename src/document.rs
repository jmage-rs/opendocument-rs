use std::collections::BTreeMap;
use std::convert::TryInto;

pub struct Document {
    pub files: BTreeMap<String, Vec<u8>>,
    pub content_xml: Option<xmltree::Element>,
    pub styles_xml: Option<xmltree::Element>,
}

impl Document {
    fn null_document() -> Document {
        Document {
            files: Default::default(),
            content_xml: None,
            styles_xml: None,
        }
    }

    pub fn load_from_path(path: impl AsRef<std::path::Path>) -> std::io::Result<Document> {
        let mut file = std::fs::File::open(path)?;
        let len = file
            .metadata()
            .map(|x| x.len().try_into().unwrap_or(8192))
            .unwrap_or(8192);
        let mut data: Vec<u8> = Vec::with_capacity(len);
        std::io::Read::read_to_end(&mut file, &mut data)?;
        Self::load_from_memory(&data)
    }

    pub fn load_from_memory(memory: &[u8]) -> std::io::Result<Document> {
        let cursor = std::io::Cursor::new(memory);
        let mut zip = zip::ZipArchive::new(cursor)?;
        let file_count = zip.len();
        let mut document = Self::null_document();
        for i in 0..file_count {
            let mut file = zip.by_index(i)?;
            let file_name = file.name().to_string();
            let file_len = file
                .size()
                .try_into()
                .map_err(|x| std::io::Error::new(std::io::ErrorKind::InvalidData, x))?;
            let mut file_bytes = Vec::with_capacity(file_len);
            std::io::Read::read_to_end(&mut file, &mut file_bytes)?;

            match file_name.as_str() {
                "content.xml" => {
                    document.content_xml =
                        Some(xmltree::Element::parse(&file_bytes[..]).map_err(|x| {
                            std::io::Error::new(std::io::ErrorKind::InvalidData, x)
                        })?);
                    continue;
                }
                "styles_xml" => {
                    document.styles_xml =
                        Some(xmltree::Element::parse(&file_bytes[..]).map_err(|x| {
                            std::io::Error::new(std::io::ErrorKind::InvalidData, x)
                        })?);
                    continue;
                }
                _ => (),
            }

            document.files.insert(file_name, file_bytes);
        }
        Ok(document)
    }

    pub fn save_to_memory(&self) -> std::io::Result<Vec<u8>> {
        let data = Vec::new();
        let cursor = std::io::Cursor::new(data);
        let mut zip = zip::ZipWriter::new(cursor);
        let mut files = self.files.clone();
        if self.content_xml.is_some() {
            let mut content_xml = Vec::new();
            self.content_xml
                .as_ref()
                .unwrap()
                .write(&mut content_xml)
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::Other, "failed to generate xml output")
                })?;
            files.insert("content.xml".to_string(), content_xml);
        }
        if self.styles_xml.is_some() {
            let mut styles_xml = Vec::new();
            self.styles_xml
                .as_ref()
                .unwrap()
                .write(&mut styles_xml)
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::Other, "failed to generate xml output")
                })?;
            files.insert("styles.xml".to_string(), styles_xml);
        }
        for (k, v) in &files {
            zip.start_file(k, zip::write::FileOptions::default())?;
            std::io::Write::write_all(&mut zip, v)?;
        }
        Ok(zip.finish()?.into_inner())
    }

    pub fn save_to_path(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let data = self.save_to_memory()?;
        let mut file = std::fs::File::create(path)?;
        std::io::Write::write_all(&mut file, &data)?;
        Ok(())
    }
}
