<script setup lang="ts">
import {onMounted, onUnmounted, ref} from 'vue'
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
import {Badge} from "@/components/ui/badge";

useTitle('Daemon — neond')
const daemonStore = useDaemonStore();

onMounted(() => daemonStore.startPolling());
onUnmounted(() => daemonStore.stopPolling());
</script>

<template>
  <div class="w-full space-y-6">
    <div v-if="daemonStore.loading && !daemonStore.state" class="text-sm text-muted-foreground">
      Loading…
    </div>

    <template v-if="daemonStore.state">
      <div class="grid md:grid-cols-3 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Storage</CardTitle>
            <CardDescription>Connected to S3 bucket</CardDescription>
          </CardHeader>
          <CardContent class="flex flex-col gap-3">

            <Table class="rounded-md border">
              <TableBody>
                <TableRow>
                  <TableCell>AWS Access Key ID</TableCell>
                  <TableCell>
                    <CodeSnippet>uszdhguisrf</CodeSnippet>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>AWS Bucket</TableCell>
                  <TableCell>
                    <CodeSnippet>pg-data</CodeSnippet>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>AWS Region</TableCell>
                  <TableCell>
                    <CodeSnippet>us-east-1</CodeSnippet>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>

            <p class="text-sm text-muted-foreground">
              To configure different data storage, relaunch daemon with appropriate config.
            </p>

            <!--            <div class="flex justify-between">-->
            <!--              <p class="text-sm">-->
            <!--                Available 37GB out of 512GB-->
            <!--              </p>-->
            <!--              <p class="text-sm text-muted-foreground">-->
            <!--                65% used-->
            <!--              </p>-->
            <!--            </div>-->

            <!--            <Progress :model-value="30" class="w-full"/>-->
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
              <p class="text-green-600 font-semibold">37 branches in sync</p>
              <p class="text-red-600">37 branches not checkpointed</p>
            </div>

            <Progress :model-value="30" class="bg-red-600" indicator-class="bg-green-600"></Progress>
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
              <TableRow>
                <TableCell>
                  Server hostname
                </TableCell>
                <TableCell>
                  <CodeSnippet>{}.bazy.local</CodeSnippet>
                </TableCell>
              </TableRow>
              <TableRow>
                <TableCell>
                  Build Version
                </TableCell>
                <TableCell>
                  <CodeSnippet>6407dbc19</CodeSnippet>
                </TableCell>
              </TableRow>

            </Table>

            <p class="text-sm">
              3 branches checkpoints are not in sync with last received WAL record. Shutdown does not guarantee
              durability of data.
            </p>
            <Button class="w-full bg-orange-500">
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
            <TableHead>TLS SNI Prefix</TableHead>
            <TableHead>Compute Status</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>First org</TableCell>
            <TableCell>Subtracker</TableCell>
            <TableCell>production</TableCell>
            <TableCell>22MB</TableCell>
            <TableCell>
              <CodeSnippet>0/15EEC10</CodeSnippet>
            </TableCell>
            <TableCell>
              <CodeSnippet>0/15EEC10</CodeSnippet>
            </TableCell>
            <TableCell>Checkpointed</TableCell>
            <TableCell>
              <CodeSnippet>slug.</CodeSnippet>
            </TableCell>
            <TableCell>Running</TableCell>
          </TableRow>
        </TableBody>
      </Table>

    </template>
  </div>
</template>