import { Placeholder } from '@tiptap/extensions'
import { Markdown } from '@tiptap/markdown'
import StarterKit from '@tiptap/starter-kit'

export function createExtensions(placeholder?: string) {
	return [
		StarterKit.configure({
			heading: { levels: [3] },
			blockquote: false,
			code: false,
			codeBlock: false,
			horizontalRule: false,
			strike: false,
			underline: false,
			link: {
				openOnClick: false,
				autolink: true,
				HTMLAttributes: {
					rel: 'noopener noreferrer nofollow',
					target: '_blank',
				},
			},
		}),
		Markdown,
		Placeholder.configure({ placeholder: placeholder ?? '' }),
	]
}

export const proseClassName =
	'[&_h3]:mt-2 [&_h3]:mb-1 [&_h3]:text-base [&_h3]:font-semibold [&_p]:my-1 [&_ul]:my-1 [&_ul]:list-disc [&_ul]:pl-5 [&_ol]:my-1 [&_ol]:list-decimal [&_ol]:pl-5 [&_a]:text-primary [&_a]:underline [&_a]:underline-offset-2 [&_strong]:font-semibold'
