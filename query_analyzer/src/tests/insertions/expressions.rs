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

fn insert_with_parameters(full_name: Vec<&'static str>, parameters: Vec<&'static str>) -> sql_ast::Statement {
    insert_with_values(
        full_name,
        vec![parameters
            .into_iter()
            .map(ident)
            .map(sql_ast::Expr::Identifier)
            .collect()],
    )
}

#[test]
fn insert_number() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema(SCHEMA)).unwrap();
    database
        .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(insert_with_values(vec![SCHEMA, TABLE], vec![vec![small_int(1)]])),
        Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            values: vec![vec![Some(StaticUntypedTree::Item(StaticUntypedItem::Const(
                UntypedValue::Number(BigDecimal::from(1))
            )))]],
        })))
    );
}

#[test]
fn insert_string() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema(SCHEMA)).unwrap();
    database
        .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::char(5))]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(insert_with_values(vec![SCHEMA, TABLE], vec![vec![string("str")]])),
        Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            values: vec![vec![Some(StaticUntypedTree::Item(StaticUntypedItem::Const(
                UntypedValue::String("str".to_owned())
            )))]],
        })))
    );
}

#[test]
fn insert_boolean() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema(SCHEMA)).unwrap();
    database
        .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(insert_with_values(vec![SCHEMA, TABLE], vec![vec![boolean(true)]])),
        Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            values: vec![vec![Some(StaticUntypedTree::Item(StaticUntypedItem::Const(
                UntypedValue::Bool(Bool(true))
            )))]],
        })))
    );
}

#[test]
fn insert_null() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema(SCHEMA)).unwrap();
    database
        .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(insert_with_values(vec![SCHEMA, TABLE], vec![vec![null()]])),
        Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            values: vec![vec![Some(StaticUntypedTree::Item(StaticUntypedItem::Const(
                UntypedValue::Null
            )))]],
        })))
    );
}

#[test]
fn insert_identifier() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema(SCHEMA)).unwrap();
    database
        .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(insert_with_values(
            vec![SCHEMA, TABLE],
            vec![vec![sql_ast::Expr::Identifier(ident("col"))]]
        )),
        Err(AnalysisError::column_cant_be_referenced(&"col"))
    );
}

#[test]
fn insert_into_table_with_parameters() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema(SCHEMA)).unwrap();
    database
        .execute(create_table(
            SCHEMA,
            TABLE,
            vec![("col_1", SqlType::small_int()), ("col_2", SqlType::small_int())],
        ))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(insert_with_parameters(vec![SCHEMA, TABLE], vec!["$1", "$2"])),
        Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            values: vec![vec![
                Some(StaticUntypedTree::Item(StaticUntypedItem::Param(0))),
                Some(StaticUntypedTree::Item(StaticUntypedItem::Param(1)))
            ]],
        })))
    );
}

#[test]
fn insert_into_table_with_parameters_and_values() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema(SCHEMA)).unwrap();
    database
        .execute(create_table(
            SCHEMA,
            TABLE,
            vec![("col_1", SqlType::small_int()), ("col_2", SqlType::small_int())],
        ))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(insert_with_values(
            vec![SCHEMA, TABLE],
            vec![vec![
                sql_ast::Expr::Identifier(ident("$1")),
                sql_ast::Expr::Value(number(1))
            ]]
        )),
        Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            values: vec![vec![
                Some(StaticUntypedTree::Item(StaticUntypedItem::Param(0))),
                Some(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Number(
                    BigDecimal::from(1)
                ))))
            ]],
        })))
    );
}

#[cfg(test)]
mod multiple_values {
    use data_manipulation_untyped_tree::{StaticUntypedItem, StaticUntypedTree, UntypedValue};

    use super::*;

    fn insert_value_as_expression_with_operation(
        left: sql_ast::Expr,
        op: sql_ast::BinaryOperator,
        right: sql_ast::Expr,
    ) -> sql_ast::Statement {
        insert_with_values(
            vec![SCHEMA, TABLE],
            vec![vec![sql_ast::Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }]],
        )
    }

    #[test]
    fn arithmetic() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_value_as_expression_with_operation(
                sql_ast::Expr::Value(number(1)),
                sql_ast::BinaryOperator::Plus,
                sql_ast::Expr::Value(number(1))
            )),
            Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                values: vec![vec![Some(StaticUntypedTree::Operation {
                    left: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Number(
                        BigDecimal::from(1)
                    )))),
                    op: Operation::Arithmetic(Arithmetic::Add),
                    right: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Number(
                        BigDecimal::from(1)
                    ))))
                })]],
            })))
        );
    }

    #[test]
    fn string_operation() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::var_char(255))]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_value_as_expression_with_operation(
                string("str"),
                sql_ast::BinaryOperator::StringConcat,
                string("str")
            )),
            Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                values: vec![vec![Some(StaticUntypedTree::Operation {
                    left: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::String(
                        "str".to_owned()
                    )))),
                    op: Operation::StringOp(StringOp::Concat),
                    right: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::String(
                        "str".to_owned()
                    ))))
                })]],
            })))
        );
    }

    #[test]
    fn comparison() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_value_as_expression_with_operation(
                sql_ast::Expr::Value(number(1)),
                sql_ast::BinaryOperator::Gt,
                sql_ast::Expr::Value(number(1))
            )),
            Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                values: vec![vec![Some(StaticUntypedTree::Operation {
                    left: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Number(
                        BigDecimal::from(1)
                    )))),
                    op: Operation::Comparison(Comparison::Gt),
                    right: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Number(
                        BigDecimal::from(1)
                    ))))
                })]],
            })))
        );
    }

    #[test]
    fn logical() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_value_as_expression_with_operation(
                sql_ast::Expr::Value(sql_ast::Value::Boolean(true)),
                sql_ast::BinaryOperator::And,
                sql_ast::Expr::Value(sql_ast::Value::Boolean(true)),
            )),
            Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                values: vec![vec![Some(StaticUntypedTree::Operation {
                    left: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Bool(
                        Bool(true)
                    )))),
                    op: Operation::Logical(Logical::And),
                    right: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Bool(
                        Bool(true)
                    )))),
                })]],
            })))
        );
    }

    #[test]
    fn bitwise() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_value_as_expression_with_operation(
                sql_ast::Expr::Value(number(1)),
                sql_ast::BinaryOperator::BitwiseOr,
                sql_ast::Expr::Value(number(1))
            )),
            Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                values: vec![vec![Some(StaticUntypedTree::Operation {
                    left: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Number(
                        BigDecimal::from(1)
                    )))),
                    op: Operation::Bitwise(Bitwise::Or),
                    right: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::Number(
                        BigDecimal::from(1)
                    ))))
                })]],
            })))
        );
    }

    #[test]
    fn pattern_matching() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_value_as_expression_with_operation(
                string("s"),
                sql_ast::BinaryOperator::Like,
                string("str")
            )),
            Ok(QueryAnalysis::Write(UntypedWrite::Insert(InsertQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                values: vec![vec![Some(StaticUntypedTree::Operation {
                    left: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::String(
                        "s".to_owned()
                    )))),
                    op: Operation::PatternMatching(PatternMatching::Like),
                    right: Box::new(StaticUntypedTree::Item(StaticUntypedItem::Const(UntypedValue::String(
                        "str".to_owned()
                    ))))
                })]],
            })))
        );
    }
}

#[cfg(test)]
mod not_supported_values {
    use super::*;

    #[test]
    fn national_strings() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_with_values(
                vec![SCHEMA, TABLE],
                vec![vec![sql_ast::Expr::Value(sql_ast::Value::NationalStringLiteral(
                    "str".to_owned()
                ))]]
            )),
            Err(AnalysisError::FeatureNotSupported(Feature::NationalStringLiteral))
        );
    }

    #[test]
    fn hex_strings() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_with_values(
                vec![SCHEMA, TABLE],
                vec![vec![sql_ast::Expr::Value(sql_ast::Value::HexStringLiteral(
                    "str".to_owned()
                ))]]
            )),
            Err(AnalysisError::FeatureNotSupported(Feature::HexStringLiteral))
        );
    }

    #[test]
    fn time_intervals() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema(SCHEMA)).unwrap();
        database
            .execute(create_table(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(insert_with_values(
                vec![SCHEMA, TABLE],
                vec![vec![sql_ast::Expr::Value(sql_ast::Value::Interval {
                    value: "value".to_owned(),
                    leading_field: None,
                    leading_precision: None,
                    last_field: None,
                    fractional_seconds_precision: None
                })]]
            )),
            Err(AnalysisError::FeatureNotSupported(Feature::TimeInterval))
        );
    }
}
