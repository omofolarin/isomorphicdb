package io.database

import java.sql.SQLException

class WorkingWithSchemas extends ContainersSpecification {
  def 'create schema'() {
    given:
      String createSchemaQuery = 'create schema CREATE_SCHEMA_TEST'
    when:
      boolean pgResult = pgExecute createSchemaQuery
    and:
      boolean dbResult = dbExecute createSchemaQuery
    then:
      pgResult == dbResult
  }

  def 'create schema if not exist'() {
    given:
      String createSchemaQuery = 'create schema if not exists CREATE_SCHEMA_IF_NOT_EXIST'
    when:
      boolean pgResult = pgExecute createSchemaQuery
    and:
      boolean dbResult = dbExecute createSchemaQuery
    then:
      pgResult == dbResult
  }

  def 'create schema with the same name'() {
    given:
      String createSchemaQuery = 'create schema WITH_THE_SAME_NAME'
    when:
      SQLException pgError
      try {
        pgExecute createSchemaQuery
        pgExecute createSchemaQuery
      } catch (SQLException e) {
        pgError = e
      }
    and:
      SQLException dbError
      try {
        dbExecute createSchemaQuery
        dbExecute createSchemaQuery
      } catch (SQLException e) {
        dbError = e
      }
    then:
      pgError.getErrorCode() == dbError.getErrorCode()
  }

  def 'drop schema'() {
    given:
      String createSchemaQuery = 'create schema CREATE_SCHEMA_TO_DROP'
      String dropSchemaQuery = 'drop schema CREATE_SCHEMA_TO_DROP'
    and:
      pgExecute createSchemaQuery
      dbExecute createSchemaQuery
    when:
      boolean pgResult = pgExecute dropSchemaQuery
    and:
      boolean dbResult = dbExecute dropSchemaQuery
    then:
      pgResult == dbResult
  }

  def 'drop schema if exists'() {
    given:
      String dropSchemaIfExistsQuery = 'drop schema if exists DROP_SCHEMA_IF_EXISTS'
    when:
      boolean pgResult = pgExecute dropSchemaIfExistsQuery
    and:
      boolean dbResult = dbExecute dropSchemaIfExistsQuery
    then:
      pgResult == dbResult
  }

  def 'drop non existent schema'() {
    given:
      String dropNonExistentSchema = 'drop schema NON_EXISTENT_SCHEMA'
    when:
      SQLException pgError
      try {
        pgExecute dropNonExistentSchema
      } catch (SQLException e) {
        pgError = e
      }
    and:
      SQLException dbError
      try {
        dbExecute dropNonExistentSchema
      } catch (SQLException e) {
        dbError = e
      }
    then:
      pgError.errorCode == dbError.errorCode
  }
}
