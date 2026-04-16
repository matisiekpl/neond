import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { sqlApi } from '@/api/sql'
import { getAppError } from '@/api/utils'
import { quoteIdentifier } from '@/lib/sqlIdentifier'
import type { TableRef } from '@/types/models/tableRef'
import type { ExecuteSqlResponse } from '@/types/dto/executeSqlResponse'

const LIST_TABLES_SQL = `SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema') AND table_type = 'BASE TABLE' ORDER BY table_schema, table_name`

export const useSqlStore = defineStore('sql', () => {
  const tablesLoading = ref(false)
  const rowsLoading = ref(false)

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
    lsn?: string | null,
  ): Promise<ExecuteSqlResponse> {
    const safePage = Math.max(1, Math.trunc(page))
    const safePageSize = Math.max(1, Math.trunc(pageSize))
    const offset = (safePage - 1) * safePageSize
    rowsLoading.value = true
    try {
      const sql = `SELECT * FROM ${quoteIdentifier(tableRef.schema)}.${quoteIdentifier(tableRef.name)} LIMIT ${safePageSize + 1} OFFSET ${offset}`
      return await sqlApi.execute(organizationId, projectId, branchId, sql, lsn)
    } catch (error) {
      toast.error(getAppError(error))
      throw error
    } finally {
      rowsLoading.value = false
    }
  }

  return { tablesLoading, rowsLoading, listTables, fetchTableData }
})
