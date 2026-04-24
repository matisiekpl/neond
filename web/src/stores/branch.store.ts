import {defineStore} from 'pinia'
import {ref} from 'vue'
import {toast} from 'vue-sonner'
import {branchesApi} from '@/api/branches'
import {getAppError} from '@/api/utils'
import type {Branch} from '@/types/models/branch'

export const useBranchStore = defineStore('branch', () => {
    const branches = ref<Branch[]>([])
    const loading = ref(false)
    const detaching = ref(false)

    function reset(): void {
        branches.value = []
        loading.value = false
    }

    async function fetch(organizationId: string, projectId: string, silent = false): Promise<void> {
        if (!silent) loading.value = true
        try {
            branches.value = await branchesApi.list(organizationId, projectId)
        } finally {
            if (!silent) loading.value = false
        }
    }

    async function create(organizationId: string, projectId: string, name: string, parentBranchId?: string): Promise<Branch> {
        try {
            const branch = await branchesApi.create(organizationId, projectId, name, parentBranchId)
            await fetch(organizationId, projectId)
            toast.success('Branch created')
            return branch
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function update(organizationId: string, projectId: string, branchId: string, name: string): Promise<void> {
        try {
            await branchesApi.update(organizationId, projectId, branchId, name)
            await fetch(organizationId, projectId)
            toast.success('Branch renamed')
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function launchEndpoint(organizationId: string, projectId: string, branchId: string, silent: boolean = false): Promise<void> {
        try {
            await branchesApi.launch(organizationId, projectId, branchId)
            await fetch(organizationId, projectId)
            if (!silent) toast.success('Endpoint started')
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function shutdownEndpoint(organizationId: string, projectId: string, branchId: string, silent: boolean = false): Promise<void> {
        try {
            await branchesApi.shutdown(organizationId, projectId, branchId)
            await fetch(organizationId, projectId)
            if (!silent) toast.success('Endpoint stopped')
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function remove(organizationId: string, projectId: string, branchId: string): Promise<void> {
        try {
            await branchesApi.remove(organizationId, projectId, branchId)
            await fetch(organizationId, projectId)
            toast.success('Branch deleted')
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function changePassword(organizationId: string, projectId: string, branchId: string, password: string): Promise<void> {
        try {
            await branchesApi.changePassword(organizationId, projectId, branchId, password)
            await fetch(organizationId, projectId)
            toast.success('Password changed')
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function restore(organizationId: string, projectId: string, branchId: string, lsn: string): Promise<Branch> {
        try {
            const restored = await branchesApi.restore(organizationId, projectId, branchId, lsn)
            await fetch(organizationId, projectId)
            toast.success('Branch restored')
            return restored
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function resetToParent(organizationId: string, projectId: string, branchId: string): Promise<void> {
        try {
            await branchesApi.resetToParent(organizationId, projectId, branchId)
            await fetch(organizationId, projectId)
            toast.success('Branch reset to parent')
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        }
    }

    async function detachAncestor(organizationId: string, projectId: string, branchId: string): Promise<void> {
        detaching.value = true
        try {
            await branchesApi.detach(organizationId, projectId, branchId)
            await fetch(organizationId, projectId)
            toast.success('Branch detached from ancestor')
        } catch (e) {
            toast.error(getAppError(e))
            throw e
        } finally {
            detaching.value = false
        }
    }

    return {branches, loading, detaching, reset, fetch, create, update, launchEndpoint, shutdownEndpoint, remove, restore, resetToParent, detachAncestor, changePassword}
})
