<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref} from 'vue'
import {useTitle} from '@vueuse/core'
import {useDaemonStore} from '@/stores/daemon.store'
import {
  Card,
  CardContent, CardDescription,
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
import CodeSnippet from "@/elements/CodeSnippet.vue";
import {Progress} from "@/components/ui/progress";
import {Button} from "@/components/ui/button";
import {formatBytes} from '@/lib/utils'
import EndpointStatusBadge from '@/elements/EndpointStatusBadge.vue'
import DurabilityStatusBadge from '@/elements/DurabilityStatusBadge.vue'
import ShutdownDaemonDialog from '@/elements/ShutdownDaemonDialog.vue'
import {Alert, AlertDescription, AlertTitle} from '@/components/ui/alert'
import {TriangleAlert} from 'lucide-vue-next'

useTitle('Daemon — neond')
const daemonStore = useDaemonStore();
const shutdownDialogOpen = ref(false)

const inSyncCount = computed(() => {
  if (!daemonStore.state) return 0
  return daemonStore.state.mappings.filter(
    (m) => m.remote_consistent_lsn_visible === m.last_record_lsn
  ).length
})

const totalCount = computed(() => daemonStore.state?.mappings.length ?? 0)

const awaitingCount = computed(() => totalCount.value - inSyncCount.value)

const pendingShutdown = computed(() => daemonStore.state?.pending_shutdown ?? null)

const hostnameRoutingEnabled = computed(() => !!daemonStore.state?.hostname)

const checkpointTimeoutMinutes = computed(() => {
  const secs = daemonStore.state?.max_checkpoint_timeout?.secs ?? 0
  return Math.ceil(secs / 60)
})

onMounted(() => daemonStore.startPolling());
onUnmounted(() => daemonStore.stopPolling());
</script>

<template>
  <div class="w-full space-y-6">
    <div v-if="daemonStore.loading && !daemonStore.state" class="text-sm text-muted-foreground">
      Loading…
    </div>

    <template v-if="daemonStore.state">
      <Alert v-if="pendingShutdown" variant="destructive" class="flex items-center justify-between gap-4">
        <TriangleAlert class="size-4" />
        <div class="flex-1">
          <AlertTitle>Daemon shutdown in progress</AlertTitle>
          <AlertDescription v-if="pendingShutdown.wait_for_checkpoints && awaitingCount > 0">
            Server is waiting for branches to checkpoint. Maximum checkpoint time is estimated to {{ checkpointTimeoutMinutes }} min.
          </AlertDescription>
          <AlertDescription v-else>Compute endpoints are being stopped.</AlertDescription>
        </div>
        <Button
          variant="outline"
          class="cursor-pointer shrink-0"
          :disabled="daemonStore.cancellingSubmitting"
          @click="daemonStore.cancelShutdown()"
        >
          Cancel shutdown
        </Button>
      </Alert>

      <div class="grid md:grid-cols-3 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Storage</CardTitle>
            <CardDescription v-if="daemonStore.state.storage.type === 'remote'">Connected to S3 bucket</CardDescription>
            <CardDescription v-else>Local disk storage</CardDescription>
          </CardHeader>
          <CardContent class="flex flex-col gap-3">

            <template v-if="daemonStore.state.storage.type === 'remote'">
              <Table class="rounded-md border">
                <TableBody>
                  <TableRow>
                    <TableCell>AWS Access Key ID</TableCell>
                    <TableCell>
                      <CodeSnippet>{{ daemonStore.state.storage.aws_access_key_id }}</CodeSnippet>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>AWS Bucket</TableCell>
                    <TableCell>
                      <CodeSnippet>{{ daemonStore.state.storage.bucket }}</CodeSnippet>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>AWS Region</TableCell>
                    <TableCell>
                      <CodeSnippet>{{ daemonStore.state.storage.region }}</CodeSnippet>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </template>

            <template v-else-if="daemonStore.state.storage.type === 'local'">
              <div class="flex justify-between text-sm">
                <p>{{ formatBytes(daemonStore.state.storage.used_bytes) }} used</p>
                <p class="text-muted-foreground">{{ formatBytes(daemonStore.state.storage.free_bytes) }} free</p>
              </div>
              <Progress :model-value="daemonStore.state.storage.used_percent" class="w-full"/>
            </template>

            <p class="text-sm text-muted-foreground">
              To configure different data storage, relaunch daemon with appropriate config.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Durability</CardTitle>
            <CardDescription>
              Neon stores data in layer files on pageservers, which receive the Postgres Write-Ahead Log (WAL) and
              provide high durability. Checkpoints — the process of flushing data to durable storage — don't happen
              instantly; Neon automatically determines the optimal time to run them. If needed, administrators can set a
              maximum interval between checkpoints via the Checkpoint timeout setting in branch settings.
            </CardDescription>
          </CardHeader>
          <CardContent class="flex flex-col gap-3">
            <div class="flex justify-between text-sm">
              <p class="text-green-600 font-semibold">{{ inSyncCount }} {{ inSyncCount === 1 ? 'branch' : 'branches' }}
                in sync</p>
              <p class="text-amber-600">{{ awaitingCount }} awaiting checkpoint</p>
            </div>
            <Progress
              :model-value="totalCount > 0 ? (inSyncCount / totalCount) * 100 : 0"
              class="bg-amber-500/30"
              indicator-class="bg-green-600"
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Maintenance</CardTitle>
            <CardDescription>
              Perform maintenance operations in graceful manner.
            </CardDescription>
          </CardHeader>
          <CardContent class="flex flex-col gap-3">
            <Table class="rounded-md border">
              <TableBody>
                <TableRow>
                  <TableCell>Server hostname</TableCell>
                  <TableCell>
                    <CodeSnippet>{{ daemonStore.state.hostname ?? '-' }}</CodeSnippet>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>Build Version</TableCell>
                  <TableCell>
                    <CodeSnippet>{{ daemonStore.state.build_version }}</CodeSnippet>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>

            <p v-if="awaitingCount > 0" class="text-sm">
              {{ awaitingCount }} {{ awaitingCount === 1 ? 'branch' : 'branches' }} not checkpointed against last received WAL record. Shutdown does not guarantee durability of data.
            </p>
            <p v-else class="text-sm">
              All branches had been checkpointed - shutdown guarantees durability in remote storage. Server migration is safe.
            </p>

            <Button
              class="w-full bg-orange-500 cursor-pointer"
              :disabled="!!pendingShutdown"
              @click="shutdownDialogOpen = true"
            >
              Shutdown
            </Button>
          </CardContent>
        </Card>
      </div>

      <h2 class="text-sm font-semibold">Branches</h2>
      <Table class="rounded-md border">
        <TableHeader>
          <TableRow>
            <TableHead>Organization</TableHead>
            <TableHead>Project</TableHead>
            <TableHead>Branch</TableHead>
            <TableHead>Size</TableHead>
            <TableHead>Last Received LSN</TableHead>
            <TableHead>Checkpointed LSN</TableHead>
            <TableHead>Sync Status</TableHead>
            <TableHead>Checkpoint Timeout</TableHead>
            <TableHead v-if="hostnameRoutingEnabled">TLS SNI Prefix</TableHead>
            <TableHead v-else>Port</TableHead>
            <TableHead>Compute Status</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="mapping in daemonStore.state.mappings" :key="mapping.branch_id">
            <TableCell>
              <RouterLink
                :to="{ name: 'projects.list', params: { organizationId: mapping.organization_id } }"
                class="underline underline-offset-2 hover:text-foreground text-muted-foreground"
              >{{ mapping.organization_name }}</RouterLink>
            </TableCell>
            <TableCell>
              <RouterLink
                :to="{ name: 'projects.show', params: { organizationId: mapping.organization_id, projectId: mapping.project_id } }"
                class="underline underline-offset-2 hover:text-foreground text-muted-foreground"
              >{{ mapping.project_name }}</RouterLink>
            </TableCell>
            <TableCell>
              <RouterLink
                :to="{ name: 'projects.show', params: { organizationId: mapping.organization_id, projectId: mapping.project_id } }"
                class="underline underline-offset-2 hover:text-foreground text-muted-foreground"
              >{{ mapping.branch_name }}</RouterLink>
            </TableCell>
            <TableCell class="text-xs text-muted-foreground">
              {{ formatBytes(mapping.current_logical_size) }}
            </TableCell>
            <TableCell>
              <CodeSnippet>{{ mapping.last_record_lsn }}</CodeSnippet>
            </TableCell>
            <TableCell>
              <CodeSnippet>{{ mapping.remote_consistent_lsn_visible }}</CodeSnippet>
            </TableCell>
            <TableCell>
              <DurabilityStatusBadge
                :last-record-lsn="mapping.last_record_lsn"
                :remote-consistent-lsn="mapping.remote_consistent_lsn_visible"
              />
            </TableCell>
            <TableCell class="text-xs text-muted-foreground">
              {{ mapping.checkpoint_timeout ? Math.ceil(mapping.checkpoint_timeout.secs / 60) + 'm' : '—' }}
            </TableCell>
            <TableCell v-if="hostnameRoutingEnabled">
              <CodeSnippet v-if="mapping.sni">{{ mapping.slug }}.</CodeSnippet>
              <span v-else class="text-xs text-muted-foreground">—</span>
            </TableCell>
            <TableCell v-else>
              <CodeSnippet v-if="mapping.port">{{ mapping.port }}</CodeSnippet>
              <span v-else class="text-xs text-muted-foreground">—</span>
            </TableCell>
            <TableCell>
              <EndpointStatusBadge :status="mapping.endpoint_status"/>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

    </template>

    <ShutdownDaemonDialog
      v-model:open="shutdownDialogOpen"
      :awaiting-count="awaitingCount"
    />
  </div>
</template>