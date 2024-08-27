use jmespath;
use jmespath::{Expression, JmespathError, ToJmespath, Runtime};


pub trait Matches {
    fn matches<T: ToJmespath>(&self, data: T) -> Result<bool, JmespathError>;
}

pub enum Filter<'a> {
    OrFilter(Vec<FilterRule<'a>>)
}
impl<'a> Matches for Filter<'a> {
    fn matches<T: ToJmespath>(&self, data: T) -> Result<bool, JmespathError> {
        match self {
            Self::OrFilter(filter_rules) => {
                let data = data.to_jmespath()?;
                let result = filter_rules.iter()
                    .any(|filter| {
                        filter.matches(&data)
                            .expect("Error matching filter!")
                    });
                Ok( result )
            }
        }
    }
}


#[derive(Clone)]
pub enum FilterRule<'a> {
    Jmes(Expression<'a>)
}
impl <'a>FilterRule<'a> {
    pub fn from_jmes(pattern: &'a str) -> Result<Self, JmespathError> {
        let expression = jmespath::compile(pattern)?;
        Ok( Self::Jmes(expression) )
    }

    pub fn from_jmes_w_runtime(pattern: &'a str, runtime: &'a Runtime) -> Result<Self, JmespathError> {
        let expression = runtime.compile(pattern)?;
        Ok( Self::Jmes(expression) )
    }
}
impl <'a> Matches for FilterRule<'a> {
    fn matches<T: ToJmespath>(&self, data: T) -> Result<bool, JmespathError> {
        match self {
            Self::Jmes(expression) => Ok(
                expression.search(data)?
                    .is_truthy()
            )
        }
    }
}