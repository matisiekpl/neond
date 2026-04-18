<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useOrganizationStore } from '@/stores/organization.store'
import { useSqlStore } from '@/stores/sql.store'
import EndpointGate from '@/elements/EndpointGate.vue'
import SqlEditor from '@/elements/SqlEditor.vue'
import SqlResultTable from '@/elements/SqlResultTable.vue'

const route = useRoute()
const organizationStore = useOrganizationStore()
const sqlStore = useSqlStore()

const projectId = computed(() => route.params.projectId as string)
const branchId = computed(() => route.params.branchId as string)
const organizationId = computed(() => organizationStore.selectedOrganizationId)

const sql = ref('')

async function runQuery() {
  if (!organizationId.value || !sql.value.trim()) return
  await sqlStore.execute(organizationId.value, projectId.value, branchId.value, sql.value)
}
</script>

<template>
  <EndpointGate
    v-if="organizationId"
    :organization-id="organizationId"
    :project-id="projectId"
    :branch-id="branchId"
  >
    <div class="grid grid-rows-[1fr_1fr] gap-6 h-full">
      <div class="border rounded-lg overflow-hidden flex flex-col">
        <SqlEditor v-model="sql" :loading="sqlStore.executeLoading" @run="runQuery" @cancel="sqlStore.cancelExecute" />
      </div>
      <div class="border rounded-lg overflow-hidden flex flex-col">
        <SqlResultTable :result="sqlStore.result" :loading="sqlStore.executeLoading" />
      </div>
    </div>
  </EndpointGate>
</template>