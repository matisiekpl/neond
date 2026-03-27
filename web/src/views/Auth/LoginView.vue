<script setup lang="ts">
import {Label} from '@/components/ui/label'
import {Button} from '@/components/ui/button'
import {Input} from '@/components/ui/input'
import {Spinner} from '@/components/ui/spinner'
import {useAuthStore} from '@/stores/auth.store.ts'
import {onMounted} from 'vue'
import {useTitle} from '@vueuse/core'
import {useRoute} from 'vue-router'

useTitle('Login — neond')
const authStore = useAuthStore()
const route = useRoute()

onMounted(() => {
  authStore.check()
})
</script>

<template>
  <div class="flex h-screen">
    <div class="m-auto flex flex-col gap-4 w-[330px]">
      <Label class="text-3xl">Login to neond</Label>
      <Label class="text-muted-foreground">Welcome back</Label>

      <form class="flex flex-col gap-4" @submit.prevent="authStore.login()">
        <div class="flex flex-col gap-2">
          <Label>Email</Label>
          <Input
            type="email"
            v-model="authStore.email"
            :disabled="!!route.query.email"
            autofocus
            required
            autocomplete="email"
          />
        </div>

        <div class="flex flex-col gap-2">
          <Label>Password</Label>
          <Input
            type="password"
            v-model="authStore.password"
            required
            autocomplete="current-password"
          />
        </div>

        <Button type="submit" class="w-full" :disabled="authStore.loading">
          <Spinner v-if="authStore.loading" class="mr-2" />
          Sign in
        </Button>

        <span class="text-muted-foreground text-sm text-center">
          Don't have an account yet?
          <RouterLink
            :to="{name: 'register'}"
            class="underline hover:text-foreground"
          >
            Sign up here
          </RouterLink>
        </span>
      </form>
    </div>
  </div>
</template>
