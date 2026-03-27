import api from '@/api/base.ts'
import type {Organization} from '@/types/models/organization.ts'

async function list(): Promise<Organization[]> {
  const response = await api.get('organizations')
  return response.data as Organization[]
}

async function create(name: string): Promise<Organization> {
  const response = await api.post('organizations', {name})
  return response.data as Organization
}

async function get(orgId: string): Promise<Organization> {
  const response = await api.get(`organizations/${orgId}`)
  return response.data as Organization
}

async function update(orgId: string, name: string): Promise<Organization> {
  const response = await api.put(`organizations/${orgId}`, {name})
  return response.data as Organization
}

async function remove(orgId: string): Promise<void> {
  await api.delete(`organizations/${orgId}`)
}

export const organizationsApi = {
  list,
  create,
  get,
  update,
  remove,
}
