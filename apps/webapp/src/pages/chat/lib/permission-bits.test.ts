import { describe, expect, it } from 'vitest'
import { readBitState, withBitState } from '#/pages/chat/lib/permission-bits'

describe('readBitState', () => {
	it('reads inherit when the bit is set in neither mask', () => {
		expect(readBitState(0, 0, 'VIEW_CHANNEL')).toBe('inherit')
	})

	it('reads allow when the bit is set in allow', () => {
		expect(readBitState(32, 0, 'VIEW_CHANNEL')).toBe('allow')
	})

	it('reads deny when the bit is set in deny', () => {
		expect(readBitState(0, 32, 'VIEW_CHANNEL')).toBe('deny')
	})

	it('reads each bit independently', () => {
		// allow=SEND_MESSAGES(64), deny=VIEW_CHANNEL(32)
		expect(readBitState(64, 32, 'VIEW_CHANNEL')).toBe('deny')
		expect(readBitState(64, 32, 'SEND_MESSAGES')).toBe('allow')
	})
})

describe('withBitState', () => {
	it('sets a bit to allow, clearing it from deny', () => {
		const result = withBitState(0, 32, 'VIEW_CHANNEL', 'allow')
		expect(result).toEqual({ allow: 32, deny: 0 })
	})

	it('sets a bit to deny, clearing it from allow', () => {
		const result = withBitState(32, 0, 'VIEW_CHANNEL', 'deny')
		expect(result).toEqual({ allow: 0, deny: 32 })
	})

	it('sets a bit to inherit, clearing it from both masks', () => {
		const result = withBitState(32, 0, 'VIEW_CHANNEL', 'inherit')
		expect(result).toEqual({ allow: 0, deny: 0 })
	})

	it('leaves the other bit untouched', () => {
		// SEND_MESSAGES already allowed; now deny VIEW_CHANNEL too.
		const result = withBitState(64, 0, 'VIEW_CHANNEL', 'deny')
		expect(result).toEqual({ allow: 64, deny: 32 })
	})

	it('round-trips through all three states', () => {
		let state = { allow: 0, deny: 0 }
		state = withBitState(state.allow, state.deny, 'VIEW_CHANNEL', 'allow')
		expect(readBitState(state.allow, state.deny, 'VIEW_CHANNEL')).toBe('allow')
		state = withBitState(state.allow, state.deny, 'VIEW_CHANNEL', 'deny')
		expect(readBitState(state.allow, state.deny, 'VIEW_CHANNEL')).toBe('deny')
		state = withBitState(state.allow, state.deny, 'VIEW_CHANNEL', 'inherit')
		expect(readBitState(state.allow, state.deny, 'VIEW_CHANNEL')).toBe(
			'inherit',
		)
	})
})
