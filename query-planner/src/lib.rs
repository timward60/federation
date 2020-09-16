#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate derive_builder;

pub use crate::builder::build_query_plan;
use crate::model::QueryPlan;
use graphql_parser::{parse_query, parse_schema, schema, ParseError};

#[macro_use]
mod macros;
mod autofrag;
mod builder;
mod consts;
mod context;
mod federation;
mod groups;
mod helpers;
pub mod model;
mod visitors;

#[derive(Debug)]
pub enum QueryPlanError {
    FailedParsingSchema(ParseError),
    FailedParsingQuery(ParseError),
    InvalidQuery(&'static str),
}

pub type Result<T> = std::result::Result<T, QueryPlanError>;

pub struct QueryPlanner<'s> {
    schema: schema::Document<'s>,
}

impl<'s> QueryPlanner<'s> {
    pub fn new(schema: &'s str) -> QueryPlanner<'s> {
        let schema = parse_schema(schema).expect("failed parsing schema");
        QueryPlanner { schema }
    }

    pub fn plan(&self, query: &str, options: QueryPlanningOptions) -> Result<QueryPlan> {
        let query = parse_query(query).expect("failed parsing query");
        build_query_plan(&self.schema, &query, options)
    }
}

// NB: By deriving Builder (using the derive_builder crate) we automatically implement
// the builder pattern for arbitrary structs.
// simple #[derive(Builder)] will generate a FooBuilder for your struct Foo with all setter-methods and a build method.
#[derive(Default, Builder, Debug)]
pub struct QueryPlanningOptions {
    auto_fragmentization: bool,
}

#[cfg(test)]
mod tests {
    use crate::model::QueryPlan;
    use crate::{QueryPlanner, QueryPlanningOptionsBuilder};
    use gherkin_rust::Feature;
    use gherkin_rust::StepType;
    use std::fs::{read_dir, read_to_string};
    use std::path::PathBuf;

    macro_rules! get_step {
        ($scenario:ident, $typ:pat) => {
            $scenario
                .steps
                .iter()
                .find(|s| matches!(s.ty, $typ))
                .unwrap()
                .docstring
                .as_ref()
                .unwrap()
        };
    }

    /// This test looks over all directorys under tests/features and finds "csdl.graphql" in
    /// each of those directories. It runs all of the .feature cases in that directory against that schema.
    /// To add test cases against new schemas, create a sub directory under "features" with the new schema
    /// and new .feature files.
    #[test]
    fn test_all_feature_files() {
        // If debugging with IJ, use `read_dir("query-planner/tests/features")`
        // let dirs = read_dir("query-planner/tests/features")
        let dirs = read_dir(PathBuf::from("tests").join("features"))
            .unwrap()
            .map(|res| res.map(|e| e.path()).unwrap())
            .filter(|d| d.is_dir());

        for dir in dirs {
            let schema = read_to_string(dir.join("csdl.graphql")).unwrap();
            let planner = QueryPlanner::new(&schema);
            let feature_paths = read_dir(dir)
                .unwrap()
                .map(|res| res.map(|e| e.path()).unwrap())
                .filter(|e| {
                    if let Some(d) = e.extension() {
                        d == "feature"
                    } else {
                        false
                    }
                });

            for path in feature_paths {
                let feature = read_to_string(&path).unwrap();

                let feature = match Feature::parse(feature) {
                    Result::Ok(feature) => feature,
                    Result::Err(e) => panic!("Unparseable .feature file {:?} -- {}", &path, e),
                };

                for scenario in feature.scenarios {
                    let query: &str = get_step!(scenario, StepType::Given);
                    let expected_str: &str = get_step!(scenario, StepType::Then);
                    let expected: QueryPlan = serde_json::from_str(&expected_str).unwrap();

                    let auto_fragmentization = scenario
                        .steps
                        .iter()
                        .any(|s| matches!(s.ty, StepType::When));
                    let options = QueryPlanningOptionsBuilder::default()
                        .auto_fragmentization(auto_fragmentization)
                        .build()
                        .unwrap();
                    let result = planner.plan(query, options).unwrap();
                    assert_eq!(result, expected);
                }
            }
        }
    }
}
