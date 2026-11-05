const MENTION_PATTERN = /(@\p{Lu}\p{L}+(?: \p{Lu}\p{L}+)?)/u

export interface TextSegment {
	key: string
	text: string
	isMention: boolean
}

export function splitMentions(text: string): TextSegment[] {
	let offset = 0
	return text
		.split(MENTION_PATTERN)
		.map((part, index) => {
			const segment = {
				key: String(offset),
				text: part,
				isMention: index % 2 === 1,
			}
			offset += part.length
			return segment
		})
		.filter((segment) => segment.text !== '')
}
