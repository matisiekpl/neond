<script setup lang="ts">
import type { SwitchRootEmits, SwitchRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SwitchRoot,
  SwitchThumb,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<SwitchRootProps & {
  class?: HTMLAttributes['class']
  size?: 'sm' | 'default'
}>(), {
  size: 'default',
})

const emits = defineEmits<SwitchRootEmits>()

const delegatedProps = reactiveOmit(props, 'class', 'size')

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <SwitchRoot
    v-slot="slotProps"
    data-slot="switch"
    :data-size="size"
    v-bind="forwarded"
    :class="cn(
      'peer group/switch relative inline-flex shrink-0 items-center rounded-full border border-transparent shadow-xs outline-none transition-all focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:border-destructive aria-invalid:ring-1 aria-invalid:ring-destructive/20 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80 data-[size=default]:h-[18px] data-[size=default]:w-8 data-[size=sm]:h-[14px] data-[size=sm]:w-6 data-[disabled]:cursor-not-allowed data-[disabled]:opacity-50 after:absolute after:-inset-x-3 after:-inset-y-2',
      props.class,
    )"
  >
    <SwitchThumb
      data-slot="switch-thumb"
      class="bg-background pointer-events-none block rounded-full ring-0 shadow-lg transition-transform group-data-[size=default]/switch:size-4 group-data-[size=sm]/switch:size-3 data-[state=unchecked]:translate-x-0 data-[state=checked]:translate-x-[calc(100%-2px)] dark:data-[state=unchecked]:bg-foreground dark:data-[state=checked]:bg-primary-foreground"
    >
      <slot name="thumb" v-bind="slotProps" />
    </SwitchThumb>
  </SwitchRoot>
</template>