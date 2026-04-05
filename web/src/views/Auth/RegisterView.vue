<script setup lang="ts">
import { ref, onMounted, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTitle } from '@vueuse/core'
import { useAuthStore } from '@/stores/auth.store'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Loader2 } from 'lucide-vue-next'

useTitle('Register — neond')
const authStore = useAuthStore()
const route = useRoute()
const router = useRouter()

const name = ref('')
const email = ref('')
const password = ref('')

onMounted(() => {
  const q = route.query.email
  if (q) email.value = q as string
})

watchEffect(() => {
  if (authStore.initialized && authStore.user) {
    router.replace('/dashboard')
  }
})

async function onSubmit() {
  await authStore.register(name.value, email.value, password.value)
}
</script>

<template>
  <div v-if="!authStore.initialized" class="flex h-screen items-center justify-center">
    <Loader2 class="size-8 animate-spin" />
  </div>
  <div v-else class="flex h-screen">
    <div class="m-auto flex w-[330px] flex-col gap-4">
      <Label class="text-3xl">Create an account</Label>
      <Label class="text-muted-foreground">Get started with neond</Label>
      <form class="flex flex-col gap-4" @submit.prevent="onSubmit">
        <div class="flex flex-col gap-2">
          <Label for="register-name">Name</Label>
          <Input
            id="register-name"
            type="text"
            v-model="name"
            autocomplete="name"
            required
          />
        </div>
        <div class="flex flex-col gap-2">
          <Label for="register-email">Email</Label>
          <Input
            id="register-email"
            type="email"
            v-model="email"
            :disabled="!!route.query.email"
            autocomplete="email"
            required
          />
        </div>
        <div class="flex flex-col gap-2">
          <Label for="register-password">Password</Label>
          <Input
            id="register-password"
            type="password"
            v-model="password"
            autocomplete="new-password"
            required
          />
        </div>
        <Button type="submit" class="w-full cursor-pointer" :disabled="authStore.loading">
          <Loader2 v-if="authStore.loading" class="mr-2 size-4 animate-spin" />
          Sign up
        </Button>
        <span class="text-center text-sm text-muted-foreground">
          Already have an account?
          <RouterLink to="/login" class="underline hover:text-foreground">
            Sign in here
          </RouterLink>
        </span>
      </form>
    </div>
  </div>
</template>
