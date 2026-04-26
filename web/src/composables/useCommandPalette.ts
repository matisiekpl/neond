import { ref } from 'vue'
import { useMagicKeys, whenever } from '@vueuse/core'

const open = ref(false)

const { Meta_K, Ctrl_K } = useMagicKeys()

whenever(Meta_K, () => {
  open.value = !open.value
})

whenever(Ctrl_K, () => {
  open.value = !open.value
})

export function useCommandPalette() {
  return { open }
}
