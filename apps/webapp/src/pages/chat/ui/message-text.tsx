import { cn } from '#/lib/utils'
import { splitMentions } from '#/pages/chat/lib/mentions'

export function MessageText({
	text,
	currentUserName,
}: {
	text: string
	currentUserName?: string
}) {
	return (
		<p className="whitespace-pre-wrap break-words text-sm leading-relaxed">
			{splitMentions(text).map((segment) =>
				segment.isMention ? (
					<span
						key={segment.key}
						className={cn(
							'rounded-sm px-1 font-medium',
							currentUserName && segment.text === `@${currentUserName}`
								? 'bg-highlight/25 font-semibold text-foreground'
								: 'bg-accent text-accent-foreground',
						)}
					>
						{segment.text}
					</span>
				) : (
					<span key={segment.key}>{segment.text}</span>
				),
			)}
		</p>
	)
}
