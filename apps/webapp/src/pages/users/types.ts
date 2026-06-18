import type { User } from '#/hooks/use-users'

export interface UserFormValues {
	email: string
	username: string
	name: string
	enabled: boolean
}

export interface CreateUserFormValues {
	email: string
	username: string
	name: string
	sendInviteEmail: boolean
}

export function userDisplayName(user: User): string {
	return user.name ?? user.username
}

export function userInitials(user: User): string {
	const source = user.name ?? user.username
	return (
		source
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase() ?? '')
			.join('') || 'U'
	)
}

export function userToForm(user: User): UserFormValues {
	return {
		email: user.email,
		username: user.username,
		name: user.name ?? '',
		enabled: user.enabled,
	}
}

export const EMPTY_CREATE_FORM: CreateUserFormValues = {
	email: '',
	username: '',
	name: '',
	sendInviteEmail: true,
}
