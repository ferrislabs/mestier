import { describe, expect, it } from 'vitest'
import { buildPermissionTargets } from './permission-targets'

const overwrite = (
	target_type: string,
	target_id: string | null,
	allow = 0,
	deny = 0,
) => ({
	target_type,
	target_id,
	allow,
	deny,
	created_at: '',
	updated_at: '',
})

describe('buildPermissionTargets', () => {
	it('lists @everyone first even without an overwrite', () => {
		const targets = buildPermissionTargets(
			[],
			() => undefined,
			() => undefined,
		)

		expect(targets).toEqual([
			{
				key: 'everyone',
				type: 'everyone',
				id: null,
				name: '@everyone',
				allow: 0,
				deny: 0,
			},
		])
	})

	it('names roles and members, sorted, and keeps their bits', () => {
		const targets = buildPermissionTargets(
			[
				overwrite('role', 'r2', 32, 0),
				overwrite('member', 'u1', 0, 64),
				overwrite('everyone', null, 0, 32),
			],
			(id) => (id === 'r2' ? 'Chefs de chantier' : undefined),
			(id) => (id === 'u1' ? 'Alice' : undefined),
		)

		expect(targets.map((t) => [t.key, t.name, t.allow, t.deny])).toEqual([
			['everyone', '@everyone', 0, 32],
			['member:u1', 'Alice', 0, 64],
			['role:r2', 'Chefs de chantier', 32, 0],
		])
	})
})
