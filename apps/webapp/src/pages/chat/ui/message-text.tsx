import { splitMentions } from '#/pages/chat/lib/mentions'

export function MessageText({ text }: { text: string }) {
	return (
		<p className="whitespace-pre-wrap break-words text-sm leading-relaxed">
			{splitMentions(text).map((segment) =>
				segment.isMention ? (
					<span
						key={segment.key}
						className="rounded-sm bg-accent px-1 font-medium text-accent-foreground"
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
