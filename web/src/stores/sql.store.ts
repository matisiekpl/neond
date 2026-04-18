import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { sqlApi } from '@/api/sql'
import { getAppError } from '@/api/utils'
import { quoteIdentifier } from '@/lib/sqlIdentifier'
import type { TableRef } from '@/types/models/tableRef'
import type { ExecuteSqlResponse } from '@/types/dto/executeSqlResponse'
import type { TableFilter, TableSort } from '@/types/dto/tableFilter'

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
  limit?: number,
  offset?: number,
): string {
  let sql = `SELECT * FROM ${quoteIdentifier(tableRef.schema)}.${quoteIdentifier(tableRef.name)}`
  sql += buildWhereClause(filters)
  if (sort) {
    sql += ` ORDER BY ${quoteIdentifier(sort.column)} ${sort.direction.toUpperCase()}`
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
    lsn?: string | null,
  ): Promise<ExecuteSqlResponse> {
    const safePage = Math.max(1, Math.trunc(page))
    const safePageSize = Math.max(1, Math.trunc(pageSize))
    const offset = (safePage - 1) * safePageSize
    rowsLoading.value = true
    try {
      const sql = buildTableDataSql(tableRef, filters, sort, safePageSize + 1, offset)
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
    lsn?: string | null,
  ): Promise<ExecuteSqlResponse> {
    rowsLoading.value = true
    try {
      const sql = buildTableDataSql(tableRef, filters, sort)
      return await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
    } catch (error) {
      toast.error(getAppError(error))
      throw error
    } finally {
      rowsLoading.value = false
    }
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

  return { tablesLoading, rowsLoading, executeLoading, result, listTables, fetchTableData, fetchAllTableData, execute, cancelExecute }
})
