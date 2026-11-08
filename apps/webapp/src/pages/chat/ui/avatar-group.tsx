import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

const MAX_SHOWN = 3

/** The first few people of a channel, overlapped, with how many more there
 * are. It opens the full list. */
export function AvatarGroup({
	members,
	onOpen,
}: {
	members: readonly { id: string; name: string; hue: number }[]
	onOpen: () => void
}) {
	if (members.length === 0) return null
	const shown = members.slice(0, MAX_SHOWN)
	const rest = members.length - shown.length

	return (
		<button
			type="button"
			onClick={onOpen}
			aria-label={
				members.length > 1
					? `Voir les ${members.length} membres du canal`
					: 'Voir le membre du canal'
			}
			className="flex shrink-0 items-center -space-x-1.5 rounded-lg outline-none transition-opacity hover:opacity-80 focus-visible:ring-2 focus-visible:ring-ring"
		>
			{shown.map((member) => (
				<PersonAvatar
					key={member.id}
					name={member.name}
					hue={member.hue}
					size="xs"
					className="rounded-lg ring-2 ring-background"
				/>
			))}
			{rest > 0 ? (
				<span
					aria-hidden
					className="tabular relative flex size-5 items-center justify-center rounded-lg bg-muted font-semibold text-[9px] text-muted-foreground ring-2 ring-background"
				>
					+{rest}
				</span>
			) : null}
		</button>
	)
}
