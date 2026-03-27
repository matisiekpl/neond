import type {AxiosError} from 'axios'

export function getAppError(err: any): string {
  const message = (((err as AxiosError).response!.data! as any).message) as string
  return String(message).charAt(0).toUpperCase() + String(message).slice(1)
}
