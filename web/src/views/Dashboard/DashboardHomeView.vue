<script setup lang="ts">
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from '@/components/ui/card'
import {useTitle} from '@vueuse/core'

useTitle('Dashboard — neond')
</script>

<template>
  <div class="space-y-4">
    <Card>
      <CardHeader>
        <CardTitle>Dashboard</CardTitle>
        <CardDescription>Welcome to your workspace.</CardDescription>
      </CardHeader>
      <CardContent class="text-sm text-muted-foreground">
        Use the sidebar to switch organizations or open organization settings.
      </CardContent>
    </Card>
  </div>
</template>
