use std::path::{Path, PathBuf};
use std::io::Cursor;
use jmespath::Runtime;
use walkdir::WalkDir;
use evtx::{EvtxParser, ParserSettings};
use polars::prelude::{DataFrame, JsonReader, SerReader};
use serde_json::{json, Map, Value};
use crate::filter::{Filter, Matches};
use crate::transformer::DocumentTransformer;
use crate::errors::CustomError;

pub struct EvtxHandler<'a> {
    pub source: PathBuf,
    pub filter: Option<Filter<'a>>,
    pub transformer: DocumentTransformer<'a>
}
impl <'a> EvtxHandler<'a> {
    pub fn from_source(source: impl AsRef<Path>) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            filter: None,
            transformer: DocumentTransformer::empty()
        }
    }

    /// Set a filter.
    pub fn with_filter(mut self, filter: Filter<'a>) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set the document transformer
    pub fn with_transformer(mut self, transformer: DocumentTransformer<'a>) -> Self {
        self.transformer = transformer;
        self
    }

    pub fn add_output_column(self, name: impl AsRef<str>, pattern: &'a str) -> Result<Self, CustomError> {
        self.add_transformer_field_from_pattern(name, pattern)
    }

    pub fn add_transformer_field_from_pattern(mut self, name: impl AsRef<str>, pattern: &'a str) -> Result<Self, CustomError> {
        self.transformer = self.transformer.add_field_from_pattern(name, pattern)?;
        Ok(self)
    }

    pub fn add_transformer_field_from_pattern_w_runtime(
        mut self,
        name: impl AsRef<str>,
        pattern: &'a str,
        runtime: &'a Runtime
    ) -> Result<Self, CustomError> {
        self.transformer = self.transformer.add_field_from_pattern_w_runtime(name, pattern, runtime)?;
        Ok(self)
    }

    pub fn process(&self) -> Result<Vec<Map<String, Value>>, CustomError> {
        if self.source.is_dir() {
            self._process_evtx_folder(&self.source)
        } else {
            self._process_evtx_file(&self.source)
        }
    }

    /// Process a folder that contains .evtx files
    fn _process_evtx_folder(
        &self, 
        source_folder: impl AsRef<Path>
    ) -> Result<Vec<Map<String, Value>>, CustomError> {
        let mut results = Vec::new();
        for entry_result in WalkDir::new(source_folder) {
            match entry_result {
                Ok(entry) => {
                    let entry_path = entry.path();
                    if !entry_path.is_file() {
                        continue;
                    }

                    if let Some(ext) = entry_path.extension(){
                        let ext_lc = ext.to_string_lossy().to_lowercase();
                        if ext_lc == "evtx" {
                            let r = self._process_evtx_file(
                                entry_path
                            )?;
                            results.extend(r);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error retrieving DirEntry: {:?}", e);
                }
            }
        }

        Ok(results)
    }

    /// Process a single .evtx file
    fn _process_evtx_file(
        &self,
        source_file: impl AsRef<Path>
    ) -> Result<Vec<Map<String, Value>>, CustomError> {
        let parser_settings = ParserSettings::new()
            .separate_json_attributes(true);

        let mut parser = EvtxParser::from_path(&source_file)
            .map_err(
                |e|
                CustomError::general_error(format!("Failed to open EVTX file {:?}: {:?}", source_file.as_ref(), e))
            )?
            .with_configuration(parser_settings);

            let mut results = Vec::new();
            for record_result in parser.records_json_value() {
                match record_result {
                    Ok(record) => {
                        let value = record.data;
                        if let Some(filter) = &self.filter {
                            if filter.matches(&value)? {
                                let new_doc = self.transformer.get_map(&value)?;
                                results.push(new_doc);
                            }
                        } else {
                            let new_doc = self.transformer.get_map(&value)?;
                            results.push(new_doc);
                        }
                    },
                    Err(_) => {}
                }
            }

        Ok(results)
    }

    /// Parse data into a dataframe
    pub fn parse_into_dataframe(&self) -> Result<DataFrame, CustomError> {
        let transformed_records = if self.source.is_dir() {
            self.process()?
        } else {
            self.process()?
        };

        let json_str: String = json!(&transformed_records).to_string();
        let df = JsonReader::new(Cursor::new(json_str))
            .finish()?;
        Ok(df)
    }
}