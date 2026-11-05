import { type Dispatch, type SetStateAction, useState } from 'react'

export function useSyncedDraft<T>(
	stored: T,
	same: (a: T, b: T) => boolean,
): readonly [T, Dispatch<SetStateAction<T>>] {
	const [draft, setDraft] = useState(stored)
	const [seen, setSeen] = useState(stored)
	if (!same(seen, stored)) {
		setSeen(stored)
		setDraft(stored)
	}
	return [draft, setDraft]
}
