<script setup lang="ts">
import {Label} from '@/components/ui/label'
import {Button} from '@/components/ui/button'
import {Input} from '@/components/ui/input'
import {Spinner} from '@/components/ui/spinner'
import {useAuthStore} from '@/stores/auth.store.ts'
import {onMounted} from 'vue'
import {useTitle} from '@vueuse/core'
import {useRoute} from 'vue-router'

useTitle('Register — neond')
const authStore = useAuthStore()
const route = useRoute()

onMounted(() => {
  authStore.check()
})
</script>

<template>
  <div class="flex h-screen">
    <div class="m-auto flex flex-col gap-4 w-[330px]">
      <Label class="text-3xl">Create an account</Label>
      <Label class="text-muted-foreground">Get started with neond</Label>

      <form class="flex flex-col gap-4" @submit.prevent="authStore.register()">
        <div class="flex flex-col gap-2">
          <Label>Name</Label>
          <Input
            type="text"
            v-model="authStore.name"
            autofocus
            required
            autocomplete="name"
          />
        </div>

        <div class="flex flex-col gap-2">
          <Label>Email</Label>
          <Input
            type="email"
            v-model="authStore.email"
            :disabled="!!route.query.email"
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
            autocomplete="new-password"
          />
        </div>

        <Button type="submit" class="w-full cursor-pointer" :disabled="authStore.loading">
          <Spinner v-if="authStore.loading" class="mr-2" />
          Sign up
        </Button>

        <span class="text-muted-foreground text-sm text-center">
          Already have an account?
          <RouterLink
            :to="{name: 'login'}"
            class="underline hover:text-foreground"
          >
            Sign in here
          </RouterLink>
        </span>
      </form>
    </div>
  </div>
</template>
