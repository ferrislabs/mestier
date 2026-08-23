/**
 * Only the two bits the server's own overwrite resolution treats as
 * channel-scoped (`discord_overwrite::resolve_channel_permissions` unions
 * `VIEW_CHANNEL | SEND_MESSAGES` into the base before applying overwrites —
 * see its doc comment). The other bits (`MANAGE_*`) are organization-wide
 * administrative permissions; overriding them per channel isn't a case the
 * server's model supports, so this doesn't offer it.
 */
export const OVERWRITE_BITS = {
	VIEW_CHANNEL: 32,
	SEND_MESSAGES: 64,
} as const

export type OverwriteBitName = keyof typeof OVERWRITE_BITS

export type TriState = 'allow' | 'deny' | 'inherit'

export function readBitState(
	allow: number,
	deny: number,
	bit: OverwriteBitName,
): TriState {
	const value = OVERWRITE_BITS[bit]
	if ((allow & value) === value) return 'allow'
	if ((deny & value) === value) return 'deny'
	return 'inherit'
}

/** Sets one bit's tri-state within an (allow, deny) pair, leaving every
 * other bit untouched. */
export function withBitState(
	allow: number,
	deny: number,
	bit: OverwriteBitName,
	state: TriState,
): { allow: number; deny: number } {
	const value = OVERWRITE_BITS[bit]
	const nextAllow = state === 'allow' ? allow | value : allow & ~value
	const nextDeny = state === 'deny' ? deny | value : deny & ~value
	return { allow: nextAllow, deny: nextDeny }
}
