export interface Author {
	name: string
	hue: number
}

export type AuthorDirectory = ReadonlyMap<string, Author>

const UNKNOWN_AUTHOR: Author = { name: 'Ancien membre', hue: 0 }

/** A message whose author left the organization, or was posted by a webhook,
 * has no directory entry — it still has to render. */
export function resolveAuthor(
	authors: AuthorDirectory,
	userId: string | null | undefined,
): Author {
	return (userId ? authors.get(userId) : undefined) ?? UNKNOWN_AUTHOR
}
