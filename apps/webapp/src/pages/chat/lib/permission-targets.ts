import type { Overwrite } from '#/hooks/use-chat'

export type PermissionTargetType = 'everyone' | 'role' | 'member'

/** Who a channel permission applies to, with what the server stores for it. */
export interface PermissionTarget {
	key: string
	type: PermissionTargetType
	id: string | null
	name: string
	allow: number
	deny: number
}

export const EVERYONE_KEY = 'everyone'

export function targetKey(type: PermissionTargetType, id: string | null) {
	return type === 'everyone' ? EVERYONE_KEY : `${type}:${id}`
}

/** `@everyone` first, even before the server holds an overwrite for it, then
 * roles and members by name. */
export function buildPermissionTargets(
	overwrites: readonly Overwrite[],
	roleName: (id: string) => string | undefined,
	memberName: (id: string) => string | undefined,
): PermissionTarget[] {
	const everyone = overwrites.find((o) => o.target_type === 'everyone')
	const others = overwrites
		.filter((o) => o.target_type !== 'everyone' && o.target_id)
		.map((o): PermissionTarget => {
			const type = o.target_type === 'role' ? 'role' : 'member'
			const id = o.target_id as string
			const known = type === 'role' ? roleName(id) : memberName(id)
			return {
				key: targetKey(type, id),
				type,
				id,
				name: known ?? (type === 'role' ? 'Rôle inconnu' : 'Membre inconnu'),
				allow: o.allow,
				deny: o.deny,
			}
		})
		.sort((a, b) => a.name.localeCompare(b.name, 'fr'))

	return [
		{
			key: EVERYONE_KEY,
			type: 'everyone',
			id: null,
			name: '@everyone',
			allow: everyone?.allow ?? 0,
			deny: everyone?.deny ?? 0,
		},
		...others,
	]
}
