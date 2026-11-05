import { type Dispatch, type SetStateAction, useState } from 'react'
import { hasSameValue } from '#/pages/crm-settings/lib/settings-model'

export function useSyncedDraft<T>(
	stored: T,
): readonly [T, Dispatch<SetStateAction<T>>, boolean] {
	const [draft, setDraft] = useState(stored)
	const [seen, setSeen] = useState(stored)
	if (!hasSameValue(seen, stored)) {
		setSeen(stored)
		setDraft(stored)
	}
	return [draft, setDraft, !hasSameValue(draft, stored)]
}
