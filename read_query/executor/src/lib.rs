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

use catalog::{Database, SqlTable};
use data_manipulation_query_result::{QueryExecution, QueryExecutionError};
use read_query_plan::SelectPlan;
use std::sync::Arc;

pub struct ReadQueryExecutor<D: Database> {
    database: Arc<D>,
}

impl<D: Database> ReadQueryExecutor<D> {
    pub fn new(database: Arc<D>) -> ReadQueryExecutor<D> {
        ReadQueryExecutor { database }
    }

    pub fn execute(&self, select: SelectPlan) -> Result<QueryExecution, QueryExecutionError> {
        log::debug!("PLAN {:?}", select);
        if select.columns.is_empty() {
            Ok(QueryExecution::Selected(
                self.database.work_with(&select.table, |table| table.select()),
            ))
        } else {
            self.database.work_with(&select.table, |table| {
                match table.select_with_columns(select.columns.clone()) {
                    Ok(data) => Ok(QueryExecution::Selected(data)),
                    Err(column_name) => Err(QueryExecutionError::SchemaDoesNotExist(column_name)),
                }
            })
        }
    }
}
