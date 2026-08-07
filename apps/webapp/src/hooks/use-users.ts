import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const USERS_PATH = '/api/v1/users'
const USER_PATH = '/api/v1/users/{id}'

function usersKey() {
	return window.tanstackApi.get(USERS_PATH).queryKey
}

function userKey(id: string) {
	return window.tanstackApi.get(USER_PATH, { path: { id } }).queryKey
}

export function useUsers() {
	return useQuery(window.tanstackApi.get(USERS_PATH).queryOptions)
}

export function useCreateUser() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', USERS_PATH).mutationOptions,
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: usersKey() })
		},
	})
}

export function useUpdateUser() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', USER_PATH).mutationOptions,
		onSuccess: async (_data, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: usersKey() }),
				queryClient.invalidateQueries({
					queryKey: userKey(variables.path.id),
				}),
			])
		},
	})
}

export function useDeleteUser() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', USER_PATH).mutationOptions,
		onSuccess: async (_res, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: usersKey() }),
				queryClient.invalidateQueries({
					queryKey: userKey(variables.path.id),
				}),
			])
		},
	})
}

export type User = Schemas.UserResponse
export type CreateUserPayload = Schemas.CreateUserRequest
export type UpdateUserPayload = Schemas.UpdateUserRequest
