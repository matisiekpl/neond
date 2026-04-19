import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { sqlApi } from '@/api/sql'
import { getAppError } from '@/api/utils'
import { quoteIdentifier } from '@/lib/sqlIdentifier'
import type { TableRef } from '@/types/models/tableRef'
import type { ExecuteSqlResponse } from '@/types/dto/executeSqlResponse'
import type { TableFilter, TableSort } from '@/types/dto/tableFilter'
import type { RowUpdate } from '@/types/dto/rowUpdate'
import type { UpdateRowResult } from '@/types/dto/updateRowResult'

const LIST_TABLES_SQL = `SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema') AND table_type = 'BASE TABLE' ORDER BY table_schema, table_name`

function escapeSqlValue(value: string): string {
  return `'${value.replace(/'/g, "''")}'`
}

function buildWhereClause(filters: TableFilter[]): string {
  const conditions = filters
    .filter((filter) => filter.operator === 'is_null' || filter.operator === 'is_not_null' || filter.value !== '')
    .map((filter) => {
      const column = quoteIdentifier(filter.column)
      switch (filter.operator) {
        case 'equals':
          return `${column} = ${escapeSqlValue(filter.value)}`
        case 'not_equals':
          return `${column} != ${escapeSqlValue(filter.value)}`
        case 'contains':
          return `${column} ILIKE ${escapeSqlValue('%' + filter.value + '%')}`
        case 'starts_with':
          return `${column} ILIKE ${escapeSqlValue(filter.value + '%')}`
        case 'greater_than':
          return `${column} > ${escapeSqlValue(filter.value)}`
        case 'less_than':
          return `${column} < ${escapeSqlValue(filter.value)}`
        case 'is_null':
          return `${column} IS NULL`
        case 'is_not_null':
          return `${column} IS NOT NULL`
      }
    })
  return conditions.length > 0 ? ` WHERE ${conditions.join(' AND ')}` : ''
}

function buildTableDataSql(
  tableRef: TableRef,
  filters: TableFilter[],
  sort: TableSort | null,
  defaultSortColumns: string[],
  limit?: number,
  offset?: number,
): string {
  let sql = `SELECT * FROM ${quoteIdentifier(tableRef.schema)}.${quoteIdentifier(tableRef.name)}`
  sql += buildWhereClause(filters)
  if (sort) {
    sql += ` ORDER BY ${quoteIdentifier(sort.column)} ${sort.direction.toUpperCase()}`
  } else if (defaultSortColumns.length > 0) {
    sql += ` ORDER BY ${defaultSortColumns.map((column) => `${quoteIdentifier(column)} ASC`).join(', ')}`
  }
  if (limit !== undefined) {
    sql += ` LIMIT ${limit}`
  }
  if (offset !== undefined) {
    sql += ` OFFSET ${offset}`
  }
  return sql
}

export const useSqlStore = defineStore('sql', () => {
  const tablesLoading = ref(false)
  const rowsLoading = ref(false)
  const executeLoading = ref(false)
  const result = ref<ExecuteSqlResponse | null>(null)
  let abortController: AbortController | null = null

  async function listTables(
    organizationId: string,
    projectId: string,
    branchId: string,
    lsn?: string | null,
  ): Promise<TableRef[]> {
    tablesLoading.value = true
    try {
      const response = await sqlApi.execute(organizationId, projectId, branchId, LIST_TABLES_SQL, lsn)
      return response.rows.map((row) => ({ schema: row[0] ?? '', name: row[1] ?? '' }))
    } catch (error) {
      toast.error(getAppError(error))
      throw error
    } finally {
      tablesLoading.value = false
    }
  }

  async function fetchTableData(
    organizationId: string,
    projectId: string,
    branchId: string,
    tableRef: TableRef,
    page: number,
    pageSize: number,
    filters: TableFilter[],
    sort: TableSort | null,
    defaultSortColumns: string[],
    lsn?: string | null,
  ): Promise<ExecuteSqlResponse> {
    const safePage = Math.max(1, Math.trunc(page))
    const safePageSize = Math.max(1, Math.trunc(pageSize))
    const offset = (safePage - 1) * safePageSize
    rowsLoading.value = true
    try {
      const sql = buildTableDataSql(tableRef, filters, sort, defaultSortColumns, safePageSize + 1, offset)
      return await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
    } catch (error) {
      toast.error(getAppError(error))
      throw error
    } finally {
      rowsLoading.value = false
    }
  }

  async function fetchAllTableData(
    organizationId: string,
    projectId: string,
    branchId: string,
    tableRef: TableRef,
    filters: TableFilter[],
    sort: TableSort | null,
    defaultSortColumns: string[],
    lsn?: string | null,
  ): Promise<ExecuteSqlResponse> {
    rowsLoading.value = true
    try {
      const sql = buildTableDataSql(tableRef, filters, sort, defaultSortColumns)
      return await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
    } catch (error) {
      toast.error(getAppError(error))
      throw error
    } finally {
      rowsLoading.value = false
    }
  }

  async function fetchPrimaryKey(
    organizationId: string,
    projectId: string,
    branchId: string,
    tableRef: TableRef,
    lsn?: string | null,
  ): Promise<string[]> {
    const qualified = `${quoteIdentifier(tableRef.schema)}.${quoteIdentifier(tableRef.name)}`
    const sql = `SELECT a.attname FROM pg_index i JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey) WHERE i.indrelid = ${escapeSqlValue(qualified)}::regclass AND i.indisprimary ORDER BY array_position(i.indkey, a.attnum)`
    try {
      const response = await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
      return response.rows.map((row) => row[0] ?? '').filter((value) => value !== '')
    } catch (error) {
      toast.error(getAppError(error))
      throw error
    }
  }

  async function updateRows(
    organizationId: string,
    projectId: string,
    branchId: string,
    tableRef: TableRef,
    updates: RowUpdate[],
    lsn?: string | null,
  ): Promise<UpdateRowResult[]> {
    const qualified = `${quoteIdentifier(tableRef.schema)}.${quoteIdentifier(tableRef.name)}`
    const results: UpdateRowResult[] = []
    for (const update of updates) {
      const setClause = Object.entries(update.changedCells)
        .map(([column, value]) => `${quoteIdentifier(column)} = ${value === null ? 'NULL' : escapeSqlValue(value)}`)
        .join(', ')
      const whereClause = Object.entries(update.primaryKeyValues)
        .map(([column, value]) => value === null
          ? `${quoteIdentifier(column)} IS NULL`
          : `${quoteIdentifier(column)} = ${escapeSqlValue(value)}`)
        .join(' AND ')
      const sql = `UPDATE ${qualified} SET ${setClause} WHERE ${whereClause}`
      try {
        const response = await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
        results.push({ error: response.error ?? null })
      } catch (error) {
        results.push({ error: getAppError(error) })
      }
    }
    return results
  }

  async function deleteRows(
    organizationId: string,
    projectId: string,
    branchId: string,
    tableRef: TableRef,
    primaryKeyValuesList: Array<Record<string, string | null>>,
    lsn?: string | null,
  ): Promise<UpdateRowResult[]> {
    const qualified = `${quoteIdentifier(tableRef.schema)}.${quoteIdentifier(tableRef.name)}`
    const results: UpdateRowResult[] = []
    for (const primaryKeyValues of primaryKeyValuesList) {
      const whereClause = Object.entries(primaryKeyValues)
        .map(([column, value]) => value === null
          ? `${quoteIdentifier(column)} IS NULL`
          : `${quoteIdentifier(column)} = ${escapeSqlValue(value)}`)
        .join(' AND ')
      const sql = `DELETE FROM ${qualified} WHERE ${whereClause}`
      try {
        const response = await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
        results.push({ error: response.error ?? null })
      } catch (error) {
        results.push({ error: getAppError(error) })
      }
    }
    return results
  }

  async function insertRows(
    organizationId: string,
    projectId: string,
    branchId: string,
    tableRef: TableRef,
    rows: Record<string, string>[],
    lsn?: string | null,
  ): Promise<UpdateRowResult[]> {
    const qualified = `${quoteIdentifier(tableRef.schema)}.${quoteIdentifier(tableRef.name)}`
    const results: UpdateRowResult[] = []
    for (const row of rows) {
      const entries = Object.entries(row).filter(([, value]) => value !== '')
      const sql = entries.length === 0
        ? `INSERT INTO ${qualified} DEFAULT VALUES`
        : `INSERT INTO ${qualified} (${entries.map(([column]) => quoteIdentifier(column)).join(', ')}) VALUES (${entries.map(([, value]) => escapeSqlValue(value)).join(', ')})`
      try {
        const response = await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
        results.push({ error: response.error ?? null })
      } catch (error) {
        results.push({ error: getAppError(error) })
      }
    }
    return results
  }

  async function execute(
    organizationId: string,
    projectId: string,
    branchId: string,
    sql: string,
    lsn?: string | null,
  ): Promise<void> {
    abortController = new AbortController()
    executeLoading.value = true
    try {
      result.value = await sqlApi.execute(organizationId, projectId, branchId, sql, lsn, abortController.signal)
    } catch (error) {
      if ((error as { name?: string }).name !== 'CanceledError') {
        toast.error(getAppError(error))
      }
    } finally {
      executeLoading.value = false
      abortController = null
    }
  }

  function cancelExecute() {
    abortController?.abort()
  }

  return { tablesLoading, rowsLoading, executeLoading, result, listTables, fetchTableData, fetchAllTableData, fetchPrimaryKey, updateRows, insertRows, deleteRows, execute, cancelExecute }
})
