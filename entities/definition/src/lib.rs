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

use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
};
use types::SqlType;

#[derive(Debug, PartialEq)]
pub struct FullTableName((String, String));

impl FullTableName {
    pub fn schema(&self) -> &str {
        &(self.0).0
    }

    pub fn table(&self) -> &str {
        &(self.0).1
    }
}

impl<'f> Into<(&'f str, &'f str)> for &'f FullTableName {
    fn into(self) -> (&'f str, &'f str) {
        (&(self.0).0, &(self.0).1)
    }
}

impl<S: ToString> From<(&S, &S)> for FullTableName {
    fn from(tuple: (&S, &S)) -> Self {
        let (schema, table) = tuple;
        FullTableName((schema.to_string(), table.to_string()))
    }
}

impl Display for FullTableName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0 .0, self.0 .1)
    }
}

impl<'o> TryFrom<&'o sql_ast::ObjectName> for FullTableName {
    type Error = TableNamingError;

    fn try_from(object: &'o sql_ast::ObjectName) -> Result<Self, Self::Error> {
        if object.0.len() > 2 {
            Err(TableNamingError(object.to_string()))
        } else {
            let (schema_name, table_name) = if object.0.len() == 1 {
                ("public".to_lowercase(), object.0.first().unwrap().value.to_lowercase())
            } else {
                (
                    object.0.first().unwrap().value.to_lowercase(),
                    object.0.last().unwrap().value.to_lowercase(),
                )
            };
            Ok(FullTableName((schema_name, table_name)))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TableNamingError(String);

impl Display for TableNamingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to process table name '{}'", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct SchemaName(String);

impl AsRef<str> for SchemaName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl<'o> TryFrom<&'o sql_ast::ObjectName> for SchemaName {
    type Error = SchemaNamingError;

    fn try_from(object: &'o sql_ast::ObjectName) -> Result<Self, Self::Error> {
        if object.0.len() != 1 {
            Err(SchemaNamingError(object.to_string()))
        } else {
            Ok(SchemaName(object.to_string().to_lowercase()))
        }
    }
}

impl SchemaName {
    pub fn from<S: ToString>(schema_name: &S) -> SchemaName {
        SchemaName(schema_name.to_string())
    }
}

pub struct SchemaNamingError(String);

impl Display for SchemaNamingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Only unqualified schema names are supported, '{}'", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    name: String,
    sql_type: SqlType,
    ord_num: usize,
}

impl ColumnDef {
    pub fn new(name: String, sql_type: SqlType, ord_num: usize) -> ColumnDef {
        ColumnDef {
            name,
            sql_type,
            ord_num,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sql_type(&self) -> SqlType {
        self.sql_type
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.name == name
    }
}

#[derive(Debug)]
pub struct TableDef {
    schema: String,
    name: String,
    columns: Vec<ColumnDef>,
}

impl TableDef {
    pub fn new(full_table_name: &FullTableName, columns: Vec<ColumnDef>) -> TableDef {
        TableDef {
            schema: full_table_name.schema().to_owned(),
            name: full_table_name.table().to_owned(),
            columns,
        }
    }

    pub fn columns(&self) -> &[ColumnDef] {
        &self.columns
    }

    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|col| col.name.clone()).collect()
    }

    pub fn has_column(&self, column_name: &str) -> bool {
        self.columns.iter().any(|col| col.name == column_name)
    }
}
