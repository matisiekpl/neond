<script setup lang="ts">
import { ref, onMounted, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTitle } from '@vueuse/core'
import { useAuthStore } from '@/stores/auth.store'
import { useOrganizationStore } from '@/stores/organization.store'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Loader2 } from 'lucide-vue-next'

useTitle('Login — neond')
const authStore = useAuthStore()
const organizationStore = useOrganizationStore()
const route = useRoute()
const router = useRouter()

const email = ref('')
const password = ref('')

onMounted(() => {
  const q = route.query.email
  if (q) email.value = q as string
})

watchEffect(() => {
  if (authStore.initialized && authStore.user && organizationStore.selectedOrganizationId) {
    router.replace({ name: 'projects.list', params: { organizationId: organizationStore.selectedOrganizationId } })
  }
})

async function onSubmit() {
  await authStore.login(email.value, password.value)
}
</script>

<template>
  <div v-if="!authStore.initialized" class="flex h-screen items-center justify-center">
    <Loader2 class="size-8 animate-spin" />
  </div>
  <div v-else class="flex h-screen">
    <div class="m-auto flex w-[330px] flex-col gap-4">
      <Label class="text-3xl">Login to neond</Label>
      <Label class="text-muted-foreground">Welcome back</Label>
      <form class="flex flex-col gap-4" @submit.prevent="onSubmit">
        <div class="flex flex-col gap-2">
          <Label for="login-email">Email</Label>
          <Input
            id="login-email"
            type="email"
            v-model="email"
            :disabled="!!route.query.email"
            autocomplete="email"
            required
          />
        </div>
        <div class="flex flex-col gap-2">
          <Label for="login-password">Password</Label>
          <Input
            id="login-password"
            type="password"
            v-model="password"
            autocomplete="current-password"
            required
          />
        </div>
        <Button type="submit" class="w-full cursor-pointer" :disabled="authStore.loading">
          <Loader2 v-if="authStore.loading" class="mr-2 size-4 animate-spin" />
          Sign in
        </Button>
        <span class="text-center text-sm text-muted-foreground">
          Don't have an account yet?
          <RouterLink to="/register" class="underline hover:text-foreground">
            Sign up here
          </RouterLink>
        </span>
      </form>
    </div>
  </div>
</template>
