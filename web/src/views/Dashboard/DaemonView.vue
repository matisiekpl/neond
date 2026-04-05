<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useTitle } from '@vueuse/core'
import { useDaemonStore } from '@/stores/daemon.store'
import { formatBytes } from '@/lib/utils'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

useTitle('Daemon — neond')
const daemonStore = useDaemonStore()

onMounted(() => daemonStore.startPolling())
onUnmounted(() => daemonStore.stopPolling())
</script>

<template>
  <div class="w-full space-y-6">
    <div v-if="daemonStore.loading && !daemonStore.state" class="text-sm text-muted-foreground">
      Loading…
    </div>

    <template v-if="daemonStore.state">
      <Card>
        <CardHeader>
          <CardTitle>Storage</CardTitle>
        </CardHeader>
        <CardContent>
          <template v-if="daemonStore.state.storage.type === 'local'">
            <div class="space-y-3">
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Type</span>
                <span class="font-medium">Local</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Used</span>
                <span class="font-medium">{{ formatBytes(daemonStore.state.storage.used_bytes) }}</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Free</span>
                <span class="font-medium">{{ formatBytes(daemonStore.state.storage.free_bytes) }}</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Used %</span>
                <span class="font-medium">{{ daemonStore.state.storage.used_percent.toFixed(1) }}%</span>
              </div>
              <div class="h-2 w-full overflow-hidden rounded-full bg-muted">
                <div
                  class="h-full bg-primary transition-all"
                  :style="{ width: `${Math.min(daemonStore.state.storage.used_percent, 100)}%` }"
                />
              </div>
            </div>
          </template>

          <template v-else-if="daemonStore.state.storage.type === 'remote'">
            <div class="space-y-3">
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Type</span>
                <span class="font-medium">S3 Bucket</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Bucket</span>
                <span class="font-medium font-mono">{{ daemonStore.state.storage.bucket }}</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Region</span>
                <span class="font-medium font-mono">{{ daemonStore.state.storage.region }}</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">AWS Access Key ID</span>
                <span class="font-medium font-mono">{{ daemonStore.state.storage.aws_access_key_id }}</span>
              </div>
            </div>
          </template>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Active mappings</CardTitle>
        </CardHeader>
        <CardContent>
          <p v-if="daemonStore.state.mappings.length === 0" class="text-sm text-muted-foreground">
            No active endpoints.
          </p>
          <div v-else class="overflow-hidden border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Organization</TableHead>
                  <TableHead>Project</TableHead>
                  <TableHead>Branch</TableHead>
                  <TableHead>Port</TableHead>
                  <TableHead>SNI</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow v-for="(mapping, i) in daemonStore.state.mappings" :key="i">
                  <TableCell>{{ mapping.organization_name }}</TableCell>
                  <TableCell>{{ mapping.project_name }}</TableCell>
                  <TableCell>{{ mapping.branch_name }}</TableCell>
                  <TableCell class="font-mono">{{ mapping.port }}</TableCell>
                  <TableCell class="font-mono text-muted-foreground">{{ mapping.sni ?? '—' }}</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
    </template>
  </div>
</template>