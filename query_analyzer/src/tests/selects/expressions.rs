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

use data_manipulation_untyped_tree::{DynamicUntypedItem, DynamicUntypedTree, UntypedValue};

use super::*;

#[test]
fn select_all_columns_from_table() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema_ops(SCHEMA)).unwrap();
    database
        .execute(create_table_ops(SCHEMA, TABLE, vec![("col1", SqlType::integer())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(select(vec![SCHEMA, TABLE])),
        Ok(QueryAnalysis::Read(SelectQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            projection_items: vec![DynamicUntypedTree::Item(DynamicUntypedItem::Column {
                name: "col1".to_owned(),
                index: 0,
                sql_type: SqlType::integer()
            })],
        }))
    );
}

#[test]
fn select_specified_column_from_table() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema_ops(SCHEMA)).unwrap();
    database
        .execute(create_table_ops(SCHEMA, TABLE, vec![("col1", SqlType::integer())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(select_with_columns(
            vec![SCHEMA, TABLE],
            vec![sql_ast::SelectItem::UnnamedExpr(sql_ast::Expr::Identifier(ident(
                "col1"
            )))]
        )),
        Ok(QueryAnalysis::Read(SelectQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            projection_items: vec![DynamicUntypedTree::Item(DynamicUntypedItem::Column {
                name: "col1".to_owned(),
                index: 0,
                sql_type: SqlType::integer()
            })],
        }))
    );
}

#[test]
fn select_column_that_is_not_in_table() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema_ops(SCHEMA)).unwrap();
    database
        .execute(create_table_ops(SCHEMA, TABLE, vec![("col1", SqlType::integer())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(select_with_columns(
            vec![SCHEMA, TABLE],
            vec![sql_ast::SelectItem::UnnamedExpr(sql_ast::Expr::Identifier(ident(
                "col2"
            )))]
        )),
        Err(AnalysisError::column_not_found(&"col2"))
    );
}

#[test]
fn select_from_table_with_constant() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema_ops(SCHEMA)).unwrap();
    database
        .execute(create_table_ops(SCHEMA, TABLE, vec![("col1", SqlType::integer())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(select_with_columns(
            vec![SCHEMA, TABLE],
            vec![sql_ast::SelectItem::UnnamedExpr(sql_ast::Expr::Value(number(1)))],
        )),
        Ok(QueryAnalysis::Read(SelectQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            projection_items: vec![DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                UntypedValue::Number(BigDecimal::from(1))
            ))],
        }))
    );
}

#[test]
fn select_parameters_from_a_table() {
    let database = InMemoryDatabase::new();
    database.execute(create_schema_ops(SCHEMA)).unwrap();
    database
        .execute(create_table_ops(SCHEMA, TABLE, vec![("col1", SqlType::integer())]))
        .unwrap();
    let analyzer = Analyzer::new(database);

    assert_eq!(
        analyzer.analyze(select_with_columns(
            vec![SCHEMA, TABLE],
            vec![sql_ast::SelectItem::UnnamedExpr(sql_ast::Expr::Identifier(ident("$1")))],
        )),
        Ok(QueryAnalysis::Read(SelectQuery {
            full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
            projection_items: vec![DynamicUntypedTree::Item(DynamicUntypedItem::Param(0))],
        }))
    );
}

#[cfg(test)]
mod multiple_values {
    use data_manipulation_untyped_tree::{DynamicUntypedItem, DynamicUntypedTree, UntypedValue};

    use super::*;

    fn select_value_as_expression_with_operation(
        left: sql_ast::Expr,
        op: sql_ast::BinaryOperator,
        right: sql_ast::Expr,
    ) -> sql_ast::Statement {
        select_with_columns(
            vec![SCHEMA, TABLE],
            vec![sql_ast::SelectItem::UnnamedExpr(sql_ast::Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })],
        )
    }

    #[test]
    fn arithmetic() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema_ops(SCHEMA)).unwrap();
        database
            .execute(create_table_ops(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(select_value_as_expression_with_operation(
                string("1"),
                sql_ast::BinaryOperator::Plus,
                sql_ast::Expr::Value(number(1))
            )),
            Ok(QueryAnalysis::Read(SelectQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                projection_items: vec![DynamicUntypedTree::Operation {
                    left: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::String("1".to_owned())
                    ))),
                    op: Operation::Arithmetic(Arithmetic::Add),
                    right: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::Number(BigDecimal::from(1))
                    )))
                }],
            }))
        );
    }

    #[test]
    fn string_operation() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema_ops(SCHEMA)).unwrap();
        database
            .execute(create_table_ops(SCHEMA, TABLE, vec![("col", SqlType::var_char(255))]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(select_value_as_expression_with_operation(
                string("str"),
                sql_ast::BinaryOperator::StringConcat,
                string("str")
            )),
            Ok(QueryAnalysis::Read(SelectQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                projection_items: vec![DynamicUntypedTree::Operation {
                    left: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::String("str".to_owned())
                    ))),
                    op: Operation::StringOp(StringOp::Concat),
                    right: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::String("str".to_owned())
                    )))
                }],
            }))
        );
    }

    #[test]
    fn comparison() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema_ops(SCHEMA)).unwrap();
        database
            .execute(create_table_ops(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(select_value_as_expression_with_operation(
                string("1"),
                sql_ast::BinaryOperator::Gt,
                sql_ast::Expr::Value(number(1))
            )),
            Ok(QueryAnalysis::Read(SelectQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                projection_items: vec![DynamicUntypedTree::Operation {
                    left: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::String("1".to_owned())
                    ))),
                    op: Operation::Comparison(Comparison::Gt),
                    right: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::Number(BigDecimal::from(1))
                    )))
                }],
            }))
        );
    }

    #[test]
    fn logical() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema_ops(SCHEMA)).unwrap();
        database
            .execute(create_table_ops(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(select_value_as_expression_with_operation(
                sql_ast::Expr::Value(sql_ast::Value::Boolean(true)),
                sql_ast::BinaryOperator::And,
                sql_ast::Expr::Value(sql_ast::Value::Boolean(true)),
            )),
            Ok(QueryAnalysis::Read(SelectQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                projection_items: vec![DynamicUntypedTree::Operation {
                    left: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(UntypedValue::Bool(
                        Bool(true)
                    )))),
                    op: Operation::Logical(Logical::And),
                    right: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(UntypedValue::Bool(
                        Bool(true)
                    )))),
                }],
            }))
        );
    }

    #[test]
    fn bitwise() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema_ops(SCHEMA)).unwrap();
        database
            .execute(create_table_ops(SCHEMA, TABLE, vec![("col", SqlType::small_int())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(select_value_as_expression_with_operation(
                sql_ast::Expr::Value(number(1)),
                sql_ast::BinaryOperator::BitwiseOr,
                sql_ast::Expr::Value(number(1))
            )),
            Ok(QueryAnalysis::Read(SelectQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                projection_items: vec![DynamicUntypedTree::Operation {
                    left: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::Number(BigDecimal::from(1))
                    ))),
                    op: Operation::Bitwise(Bitwise::Or),
                    right: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::Number(BigDecimal::from(1))
                    )))
                }],
            }))
        );
    }

    #[test]
    fn pattern_matching() {
        let database = InMemoryDatabase::new();
        database.execute(create_schema_ops(SCHEMA)).unwrap();
        database
            .execute(create_table_ops(SCHEMA, TABLE, vec![("col", SqlType::bool())]))
            .unwrap();
        let analyzer = Analyzer::new(database);

        assert_eq!(
            analyzer.analyze(select_value_as_expression_with_operation(
                string("s"),
                sql_ast::BinaryOperator::Like,
                string("str")
            )),
            Ok(QueryAnalysis::Read(SelectQuery {
                full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                projection_items: vec![DynamicUntypedTree::Operation {
                    left: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::String("s".to_owned())
                    ))),
                    op: Operation::PatternMatching(PatternMatching::Like),
                    right: Box::new(DynamicUntypedTree::Item(DynamicUntypedItem::Const(
                        UntypedValue::String("str".to_owned())
                    )))
                }],
            }))
        );
    }
}
