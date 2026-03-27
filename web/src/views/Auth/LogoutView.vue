<script setup lang="ts">
import {useAuthStore} from '@/stores/auth.store.ts'
import {onMounted} from 'vue'

const authStore = useAuthStore()

onMounted(() => {
  authStore.logout()
})
</script>

<template>
  <div />
</template>
