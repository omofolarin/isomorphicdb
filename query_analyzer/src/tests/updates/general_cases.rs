// Copyright 2020 - present Alex Dukhno
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

#[test]
fn schema_does_not_exist() {
    let analyzer = Analyzer::new(InMemoryDatabase::new());

    assert_eq!(
        analyzer.analyze(update_statement(
            vec!["non_existent_schema", "non_existent_table"],
            vec![]
        )),
        Err(AnalysisError::schema_does_not_exist(&"non_existent_schema"))
    )
}

#[test]
fn table_does_not_exist() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema_ops(SCHEMA)).unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(update_statement(vec![SCHEMA, "non_existent"], vec![])),
        Err(AnalysisError::table_does_not_exist(format!(
            "{}.{}",
            SCHEMA, "non_existent"
        )))
    );
}

#[test]
fn table_with_unqualified_name() {
    let analyzer = Analyzer::new(InMemoryDatabase::new());
    assert_eq!(
        analyzer.analyze(update_statement(vec!["only_schema_in_the_name"], vec![])),
        Err(AnalysisError::table_does_not_exist(&"public.only_schema_in_the_name"))
    );
}

#[test]
fn table_with_unsupported_name() {
    let analyzer = Analyzer::new(InMemoryDatabase::new());
    assert_eq!(
        analyzer.analyze(update_statement(
            vec!["first_part", "second_part", "third_part", "fourth_part",],
            vec![]
        )),
        Err(AnalysisError::table_naming_error(
            &"Unable to process table name 'first_part.second_part.third_part.fourth_part'",
        ))
    );
}
