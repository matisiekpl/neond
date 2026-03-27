import api from '@/api/base.ts'
import type {OrganizationMemberUser} from '@/types/models/organizationMemberUser.ts'

async function list(orgId: string): Promise<OrganizationMemberUser[]> {
  const response = await api.get(`organizations/${orgId}/members`)
  return response.data as OrganizationMemberUser[]
}

async function addByEmail(orgId: string, email: string): Promise<void> {
  await api.post(`organizations/${orgId}/members`, {email})
}

async function remove(orgId: string, userId: string): Promise<void> {
  await api.delete(`organizations/${orgId}/members/${userId}`)
}

export const membersApi = {
  list,
  addByEmail,
  remove,
}
