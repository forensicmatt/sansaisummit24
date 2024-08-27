
use serde_json::{json, Map, Value};
use jmespath;
use jmespath::{Expression, JmespathError, ToJmespath, Rcvar, Runtime};

pub struct FieldRetriever<'a> {
    pub name: String,
    pub expression: Expression<'a>,
}
impl <'a>FieldRetriever<'a> {
    pub fn from_pattern(name: impl AsRef<str>, pattern: &'a str) -> Result<Self, JmespathError> {
        let expression = jmespath::compile(pattern)?;
        Ok( Self {
            name: name.as_ref().to_string().clone(),
            expression
        })
    }

    pub fn from_pattern_w_runtime(name: impl AsRef<str>, pattern: &'a str, runtime: &'a Runtime) -> Result<Self, JmespathError> {
        let expression = runtime.compile(pattern)?;
        Ok( Self {
            name: name.as_ref().to_string().clone(),
            expression
        })
    }

    pub fn search<T: ToJmespath>(&self, data: T) -> Result<Rcvar, JmespathError> {
        self.expression.search(data)
    }

    pub fn get_map<T: ToJmespath>(&self, data: T) -> Result<Map<String, Value>, JmespathError> {
        let value = self.expression.search(data)?;
        let mut map = Map::new();
        map.insert(self.name.clone(), json!(value));
        Ok(map)
    }
}


pub struct DocumentTransformer<'a> {
    fields: Vec<FieldRetriever<'a>>
}
impl <'a>DocumentTransformer<'a> {
    /// Create an empty DocumentTransformer
    pub fn empty() -> Self {
        Self {
            fields: Vec::new()
        }
    }

    /// Get a Map of the transformed document
    pub fn get_map<T: ToJmespath>(&self, data: T) -> Result<Map<String, Value>, JmespathError> {
        let data = data.to_jmespath()?;
        let mut map = Map::new();
        for field_retriever in &self.fields {
            let m = field_retriever.get_map(&data)?;
            map.extend(m);
        }
        Ok(map)
    }

    /// Add a field to the DocumentTransformer
    pub fn add_field_from_pattern(mut self, name: impl AsRef<str>, pattern: &'a str) -> Result<Self, JmespathError> {
        let retriever = FieldRetriever::from_pattern(name, pattern)?;
        self.fields.push(retriever);
        Ok(self)
    }

    /// Add a field to the DocumentTransformer with custom Runtime
    pub fn add_field_from_pattern_w_runtime(mut self, name: impl AsRef<str>, pattern: &'a str, runtime: &'a Runtime) -> Result<Self, JmespathError> {
        let retriever = FieldRetriever::from_pattern_w_runtime(name, pattern, runtime)?;
        self.fields.push(retriever);
        Ok(self)
    }
}