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
use data_definition_execution_plan::{ColumnInfo, CreateSchemaQuery, CreateTableQuery, DropSchemasQuery, SchemaChange};

#[cfg(test)]
mod create_schema;
#[cfg(test)]
mod create_table;
#[cfg(test)]
mod drop_statements;
