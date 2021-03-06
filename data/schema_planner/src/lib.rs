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

use data_definition_execution_plan::{
    ColumnInfo, CreateSchemaQuery, CreateTableQuery, DropSchemasQuery, DropTablesQuery, SchemaChange,
};
use data_definition_operations::{Kind, ObjectState, Record, Step, SystemObject, SystemOperation};

pub struct SystemSchemaPlanner;

impl SystemSchemaPlanner {
    pub const fn new() -> SystemSchemaPlanner {
        SystemSchemaPlanner
    }

    pub fn schema_change_plan(&self, schema_change: &SchemaChange) -> SystemOperation {
        match schema_change {
            SchemaChange::CreateSchema(CreateSchemaQuery {
                schema_name,
                if_not_exists,
            }) => {
                let mut steps = vec![];
                steps.push(Step::CheckExistence {
                    system_object: SystemObject::Schema,
                    object_name: vec![schema_name.as_ref().to_string()],
                });
                steps.push(Step::CreateFolder {
                    name: schema_name.as_ref().to_string(),
                });
                steps.push(Step::CreateRecord {
                    record: Record::Schema {
                        schema_name: schema_name.as_ref().to_string(),
                    },
                });
                SystemOperation {
                    kind: Kind::Create(SystemObject::Schema),
                    skip_steps_if: if *if_not_exists {
                        Some(ObjectState::Exists)
                    } else {
                        None
                    },
                    steps: vec![steps],
                }
            }
            SchemaChange::DropSchemas(DropSchemasQuery {
                schema_names,
                cascade,
                if_exists,
            }) => {
                let mut steps = vec![];
                for schema_name in schema_names {
                    let mut for_schema = vec![];
                    for_schema.push(Step::CheckExistence {
                        system_object: SystemObject::Schema,
                        object_name: vec![schema_name.as_ref().to_string()],
                    });
                    if *cascade {
                        for_schema.push(Step::RemoveDependants {
                            system_object: SystemObject::Schema,
                            object_name: vec![schema_name.as_ref().to_string()],
                        });
                    } else {
                        for_schema.push(Step::CheckDependants {
                            system_object: SystemObject::Schema,
                            object_name: vec![schema_name.as_ref().to_string()],
                        });
                    }
                    for_schema.push(Step::RemoveRecord {
                        record: Record::Schema {
                            schema_name: schema_name.as_ref().to_string(),
                        },
                    });
                    for_schema.push(Step::RemoveFolder {
                        name: schema_name.as_ref().to_string(),
                        only_if_empty: !*cascade,
                    });
                    steps.push(for_schema);
                }
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Schema),
                    skip_steps_if: if *if_exists { Some(ObjectState::NotExists) } else { None },
                    steps,
                }
            }
            SchemaChange::CreateTable(CreateTableQuery {
                full_table_name,
                column_defs,
                if_not_exists,
            }) => {
                let mut steps = vec![];
                steps.push(Step::CheckExistence {
                    system_object: SystemObject::Schema,
                    object_name: vec![full_table_name.schema().to_owned()],
                });
                steps.push(Step::CheckExistence {
                    system_object: SystemObject::Table,
                    object_name: vec![full_table_name.schema().to_owned(), full_table_name.table().to_owned()],
                });
                steps.push(Step::CreateFile {
                    folder_name: full_table_name.schema().to_owned(),
                    name: full_table_name.table().to_owned(),
                });
                steps.push(Step::CreateRecord {
                    record: Record::Table {
                        schema_name: full_table_name.schema().to_owned(),
                        table_name: full_table_name.table().to_owned(),
                    },
                });
                for ColumnInfo { name, sql_type } in column_defs {
                    steps.push(Step::CreateRecord {
                        record: Record::Column {
                            schema_name: full_table_name.schema().to_owned(),
                            table_name: full_table_name.table().to_owned(),
                            column_name: name.clone(),
                            sql_type: *sql_type,
                        },
                    })
                }
                SystemOperation {
                    kind: Kind::Create(SystemObject::Table),
                    skip_steps_if: if *if_not_exists {
                        Some(ObjectState::Exists)
                    } else {
                        None
                    },
                    steps: vec![steps],
                }
            }
            SchemaChange::DropTables(DropTablesQuery {
                full_table_names,
                if_exists,
                ..
            }) => {
                let mut steps = vec![];
                for full_table_name in full_table_names {
                    let mut for_table = vec![];
                    for_table.push(Step::CheckExistence {
                        system_object: SystemObject::Schema,
                        object_name: vec![full_table_name.schema().to_owned()],
                    });
                    for_table.push(Step::CheckExistence {
                        system_object: SystemObject::Table,
                        object_name: vec![full_table_name.schema().to_owned(), full_table_name.table().to_owned()],
                    });
                    for_table.push(Step::RemoveColumns {
                        schema_name: full_table_name.schema().to_owned(),
                        table_name: full_table_name.table().to_owned(),
                    });
                    for_table.push(Step::RemoveRecord {
                        record: Record::Table {
                            schema_name: full_table_name.schema().to_owned(),
                            table_name: full_table_name.table().to_owned(),
                        },
                    });
                    for_table.push(Step::RemoveFile {
                        folder_name: full_table_name.schema().to_owned(),
                        name: full_table_name.table().to_owned(),
                    });
                    steps.push(for_table);
                }
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Table),
                    skip_steps_if: if *if_exists { Some(ObjectState::NotExists) } else { None },
                    steps,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use definition::SchemaName;
    use types::SqlType;

    const SCHEMA: &str = "schema";
    const OTHER_SCHEMA: &str = "other_schema";
    const TABLE: &str = "table";
    const OTHER_TABLE: &str = "other_table";

    const QUERY_PLANNER: SystemSchemaPlanner = SystemSchemaPlanner::new();

    #[cfg(test)]
    mod schema {
        use super::*;
        use data_definition_execution_plan::{CreateSchemaQuery, DropSchemasQuery, SchemaChange};

        #[test]
        fn create() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::CreateSchema(CreateSchemaQuery {
                    schema_name: SchemaName::from(&SCHEMA),
                    if_not_exists: false,
                })),
                SystemOperation {
                    kind: Kind::Create(SystemObject::Schema),
                    skip_steps_if: None,
                    steps: vec![vec![
                        Step::CheckExistence {
                            system_object: SystemObject::Schema,
                            object_name: vec![SCHEMA.to_owned()],
                        },
                        Step::CreateFolder {
                            name: SCHEMA.to_owned()
                        },
                        Step::CreateRecord {
                            record: Record::Schema {
                                schema_name: SCHEMA.to_owned()
                            }
                        }
                    ]]
                }
            );
        }

        #[test]
        fn create_if_not_exists() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::CreateSchema(CreateSchemaQuery {
                    schema_name: SchemaName::from(&SCHEMA),
                    if_not_exists: true,
                })),
                SystemOperation {
                    kind: Kind::Create(SystemObject::Schema),
                    skip_steps_if: Some(ObjectState::Exists),
                    steps: vec![vec![
                        Step::CheckExistence {
                            system_object: SystemObject::Schema,
                            object_name: vec![SCHEMA.to_owned()],
                        },
                        Step::CreateFolder {
                            name: SCHEMA.to_owned()
                        },
                        Step::CreateRecord {
                            record: Record::Schema {
                                schema_name: SCHEMA.to_owned()
                            }
                        }
                    ]]
                }
            );
        }

        #[test]
        fn drop_single_schema() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::DropSchemas(DropSchemasQuery {
                    schema_names: vec![SchemaName::from(&SCHEMA)],
                    cascade: false,
                    if_exists: false
                })),
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Schema),
                    skip_steps_if: None,
                    steps: vec![vec![
                        Step::CheckExistence {
                            system_object: SystemObject::Schema,
                            object_name: vec![SCHEMA.to_owned()],
                        },
                        Step::CheckDependants {
                            system_object: SystemObject::Schema,
                            object_name: vec![SCHEMA.to_owned()],
                        },
                        Step::RemoveRecord {
                            record: Record::Schema {
                                schema_name: SCHEMA.to_owned()
                            }
                        },
                        Step::RemoveFolder {
                            name: SCHEMA.to_owned(),
                            only_if_empty: true,
                        }
                    ]]
                }
            );
        }

        #[test]
        fn drop_many() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::DropSchemas(DropSchemasQuery {
                    schema_names: vec![SchemaName::from(&SCHEMA), SchemaName::from(&OTHER_SCHEMA)],
                    cascade: false,
                    if_exists: false
                })),
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Schema),
                    skip_steps_if: None,
                    steps: vec![
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckDependants {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::RemoveRecord {
                                record: Record::Schema {
                                    schema_name: SCHEMA.to_owned()
                                }
                            },
                            Step::RemoveFolder {
                                name: SCHEMA.to_owned(),
                                only_if_empty: true,
                            }
                        ],
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![OTHER_SCHEMA.to_owned()],
                            },
                            Step::CheckDependants {
                                system_object: SystemObject::Schema,
                                object_name: vec![OTHER_SCHEMA.to_owned()]
                            },
                            Step::RemoveRecord {
                                record: Record::Schema {
                                    schema_name: OTHER_SCHEMA.to_owned()
                                }
                            },
                            Step::RemoveFolder {
                                name: OTHER_SCHEMA.to_owned(),
                                only_if_empty: true,
                            }
                        ]
                    ]
                }
            );
        }

        #[test]
        fn drop_many_cascade() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::DropSchemas(DropSchemasQuery {
                    schema_names: vec![SchemaName::from(&SCHEMA), SchemaName::from(&OTHER_SCHEMA)],
                    cascade: true,
                    if_exists: false
                })),
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Schema),
                    skip_steps_if: None,
                    steps: vec![
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::RemoveDependants {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()]
                            },
                            Step::RemoveRecord {
                                record: Record::Schema {
                                    schema_name: SCHEMA.to_owned()
                                }
                            },
                            Step::RemoveFolder {
                                name: SCHEMA.to_owned(),
                                only_if_empty: false,
                            }
                        ],
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![OTHER_SCHEMA.to_owned()],
                            },
                            Step::RemoveDependants {
                                system_object: SystemObject::Schema,
                                object_name: vec![OTHER_SCHEMA.to_owned()],
                            },
                            Step::RemoveRecord {
                                record: Record::Schema {
                                    schema_name: OTHER_SCHEMA.to_owned()
                                }
                            },
                            Step::RemoveFolder {
                                name: OTHER_SCHEMA.to_owned(),
                                only_if_empty: false,
                            }
                        ]
                    ]
                }
            );
        }

        #[test]
        fn drop_many_if_exists() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::DropSchemas(DropSchemasQuery {
                    schema_names: vec![SchemaName::from(&SCHEMA), SchemaName::from(&OTHER_SCHEMA)],
                    cascade: false,
                    if_exists: true
                })),
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Schema),
                    skip_steps_if: Some(ObjectState::NotExists),
                    steps: vec![
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckDependants {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::RemoveRecord {
                                record: Record::Schema {
                                    schema_name: SCHEMA.to_owned()
                                }
                            },
                            Step::RemoveFolder {
                                name: SCHEMA.to_owned(),
                                only_if_empty: true,
                            }
                        ],
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![OTHER_SCHEMA.to_owned()],
                            },
                            Step::CheckDependants {
                                system_object: SystemObject::Schema,
                                object_name: vec![OTHER_SCHEMA.to_owned()],
                            },
                            Step::RemoveRecord {
                                record: Record::Schema {
                                    schema_name: OTHER_SCHEMA.to_owned()
                                }
                            },
                            Step::RemoveFolder {
                                name: OTHER_SCHEMA.to_owned(),
                                only_if_empty: true,
                            }
                        ]
                    ]
                }
            );
        }
    }

    #[cfg(test)]
    mod table {
        use data_definition_execution_plan::{ColumnInfo, CreateTableQuery, DropTablesQuery, SchemaChange};

        use super::*;
        use definition::FullTableName;

        #[test]
        fn create_without_columns() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::CreateTable(CreateTableQuery {
                    full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                    column_defs: vec![],
                    if_not_exists: false,
                })),
                SystemOperation {
                    kind: Kind::Create(SystemObject::Table),
                    skip_steps_if: None,
                    steps: vec![vec![
                        Step::CheckExistence {
                            system_object: SystemObject::Schema,
                            object_name: vec![SCHEMA.to_owned()],
                        },
                        Step::CheckExistence {
                            system_object: SystemObject::Table,
                            object_name: vec![SCHEMA.to_owned(), TABLE.to_owned()],
                        },
                        Step::CreateFile {
                            folder_name: SCHEMA.to_owned(),
                            name: TABLE.to_owned()
                        },
                        Step::CreateRecord {
                            record: Record::Table {
                                schema_name: SCHEMA.to_owned(),
                                table_name: TABLE.to_owned(),
                            }
                        }
                    ]]
                }
            );
        }

        #[test]
        fn create_if_not_exists() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::CreateTable(CreateTableQuery {
                    full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                    column_defs: vec![],
                    if_not_exists: true,
                })),
                SystemOperation {
                    kind: Kind::Create(SystemObject::Table),
                    skip_steps_if: Some(ObjectState::Exists),
                    steps: vec![vec![
                        Step::CheckExistence {
                            system_object: SystemObject::Schema,
                            object_name: vec![SCHEMA.to_owned()],
                        },
                        Step::CheckExistence {
                            system_object: SystemObject::Table,
                            object_name: vec![SCHEMA.to_owned(), TABLE.to_owned()],
                        },
                        Step::CreateFile {
                            folder_name: SCHEMA.to_owned(),
                            name: TABLE.to_owned()
                        },
                        Step::CreateRecord {
                            record: Record::Table {
                                schema_name: SCHEMA.to_owned(),
                                table_name: TABLE.to_owned(),
                            }
                        }
                    ]]
                }
            );
        }

        #[test]
        fn create_with_columns() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::CreateTable(CreateTableQuery {
                    full_table_name: FullTableName::from((&SCHEMA, &TABLE)),
                    column_defs: vec![
                        ColumnInfo {
                            name: "col_1".to_owned(),
                            sql_type: SqlType::small_int()
                        },
                        ColumnInfo {
                            name: "col_2".to_owned(),
                            sql_type: SqlType::big_int()
                        }
                    ],
                    if_not_exists: false,
                })),
                SystemOperation {
                    kind: Kind::Create(SystemObject::Table),
                    skip_steps_if: None,
                    steps: vec![vec![
                        Step::CheckExistence {
                            system_object: SystemObject::Schema,
                            object_name: vec![SCHEMA.to_owned()],
                        },
                        Step::CheckExistence {
                            system_object: SystemObject::Table,
                            object_name: vec![SCHEMA.to_owned(), TABLE.to_owned()],
                        },
                        Step::CreateFile {
                            folder_name: SCHEMA.to_owned(),
                            name: TABLE.to_owned()
                        },
                        Step::CreateRecord {
                            record: Record::Table {
                                schema_name: SCHEMA.to_owned(),
                                table_name: TABLE.to_owned(),
                            }
                        },
                        Step::CreateRecord {
                            record: Record::Column {
                                schema_name: SCHEMA.to_string(),
                                table_name: TABLE.to_string(),
                                column_name: "col_1".to_string(),
                                sql_type: SqlType::small_int()
                            }
                        },
                        Step::CreateRecord {
                            record: Record::Column {
                                schema_name: SCHEMA.to_string(),
                                table_name: TABLE.to_string(),
                                column_name: "col_2".to_string(),
                                sql_type: SqlType::big_int()
                            }
                        }
                    ]]
                }
            );
        }

        #[test]
        fn drop_many() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::DropTables(DropTablesQuery {
                    full_table_names: vec![
                        FullTableName::from((&SCHEMA, &TABLE)),
                        FullTableName::from((&SCHEMA, &OTHER_TABLE))
                    ],
                    cascade: false,
                    if_exists: false
                })),
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Table),
                    skip_steps_if: None,
                    steps: vec![
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckExistence {
                                system_object: SystemObject::Table,
                                object_name: vec![SCHEMA.to_owned(), TABLE.to_owned()],
                            },
                            Step::RemoveColumns {
                                schema_name: SCHEMA.to_owned(),
                                table_name: TABLE.to_owned()
                            },
                            Step::RemoveRecord {
                                record: Record::Table {
                                    schema_name: SCHEMA.to_owned(),
                                    table_name: TABLE.to_owned(),
                                }
                            },
                            Step::RemoveFile {
                                folder_name: SCHEMA.to_owned(),
                                name: TABLE.to_owned()
                            }
                        ],
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckExistence {
                                system_object: SystemObject::Table,
                                object_name: vec![SCHEMA.to_owned(), OTHER_TABLE.to_owned()],
                            },
                            Step::RemoveColumns {
                                schema_name: SCHEMA.to_owned(),
                                table_name: OTHER_TABLE.to_owned()
                            },
                            Step::RemoveRecord {
                                record: Record::Table {
                                    schema_name: SCHEMA.to_owned(),
                                    table_name: OTHER_TABLE.to_owned(),
                                }
                            },
                            Step::RemoveFile {
                                folder_name: SCHEMA.to_owned(),
                                name: OTHER_TABLE.to_owned()
                            }
                        ]
                    ]
                }
            );
        }

        #[test]
        fn drop_many_cascade() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::DropTables(DropTablesQuery {
                    full_table_names: vec![
                        FullTableName::from((&SCHEMA, &TABLE)),
                        FullTableName::from((&SCHEMA, &OTHER_TABLE))
                    ],
                    cascade: true,
                    if_exists: false
                })),
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Table),
                    skip_steps_if: None,
                    steps: vec![
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckExistence {
                                system_object: SystemObject::Table,
                                object_name: vec![SCHEMA.to_owned(), TABLE.to_owned()],
                            },
                            Step::RemoveColumns {
                                schema_name: SCHEMA.to_owned(),
                                table_name: TABLE.to_owned()
                            },
                            Step::RemoveRecord {
                                record: Record::Table {
                                    schema_name: SCHEMA.to_owned(),
                                    table_name: TABLE.to_owned(),
                                }
                            },
                            Step::RemoveFile {
                                folder_name: SCHEMA.to_owned(),
                                name: TABLE.to_owned()
                            }
                        ],
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckExistence {
                                system_object: SystemObject::Table,
                                object_name: vec![SCHEMA.to_owned(), OTHER_TABLE.to_owned()],
                            },
                            Step::RemoveColumns {
                                schema_name: SCHEMA.to_owned(),
                                table_name: OTHER_TABLE.to_owned()
                            },
                            Step::RemoveRecord {
                                record: Record::Table {
                                    schema_name: SCHEMA.to_owned(),
                                    table_name: OTHER_TABLE.to_owned(),
                                }
                            },
                            Step::RemoveFile {
                                folder_name: SCHEMA.to_owned(),
                                name: OTHER_TABLE.to_owned()
                            }
                        ]
                    ]
                }
            );
        }

        #[test]
        fn drop_many_if_exists() {
            assert_eq!(
                QUERY_PLANNER.schema_change_plan(&SchemaChange::DropTables(DropTablesQuery {
                    full_table_names: vec![
                        FullTableName::from((&SCHEMA, &TABLE)),
                        FullTableName::from((&SCHEMA, &OTHER_TABLE))
                    ],
                    cascade: false,
                    if_exists: true
                })),
                SystemOperation {
                    kind: Kind::Drop(SystemObject::Table),
                    skip_steps_if: Some(ObjectState::NotExists),
                    steps: vec![
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckExistence {
                                system_object: SystemObject::Table,
                                object_name: vec![SCHEMA.to_owned(), TABLE.to_owned()],
                            },
                            Step::RemoveColumns {
                                schema_name: SCHEMA.to_owned(),
                                table_name: TABLE.to_owned(),
                            },
                            Step::RemoveRecord {
                                record: Record::Table {
                                    schema_name: SCHEMA.to_owned(),
                                    table_name: TABLE.to_owned(),
                                }
                            },
                            Step::RemoveFile {
                                folder_name: SCHEMA.to_owned(),
                                name: TABLE.to_owned()
                            }
                        ],
                        vec![
                            Step::CheckExistence {
                                system_object: SystemObject::Schema,
                                object_name: vec![SCHEMA.to_owned()],
                            },
                            Step::CheckExistence {
                                system_object: SystemObject::Table,
                                object_name: vec![SCHEMA.to_owned(), OTHER_TABLE.to_owned()],
                            },
                            Step::RemoveColumns {
                                schema_name: SCHEMA.to_owned(),
                                table_name: OTHER_TABLE.to_owned(),
                            },
                            Step::RemoveRecord {
                                record: Record::Table {
                                    schema_name: SCHEMA.to_owned(),
                                    table_name: OTHER_TABLE.to_owned(),
                                }
                            },
                            Step::RemoveFile {
                                folder_name: SCHEMA.to_owned(),
                                name: OTHER_TABLE.to_owned()
                            }
                        ]
                    ]
                }
            );
        }
    }
}
