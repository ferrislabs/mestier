import { useEffect, useState } from 'react'
import type { AdminListParams } from '#/hooks/use-admin'

const SEARCH_DELAY_MS = 300
const PER_PAGE = 20

export function useAdminList() {
	const [input, setInput] = useState('')
	const [search, setSearch] = useState('')
	const [page, setPage] = useState(1)

	useEffect(() => {
		const timer = setTimeout(() => {
			setSearch((current) => {
				const next = input.trim()
				if (next !== current) setPage(1)
				return next
			})
		}, SEARCH_DELAY_MS)
		return () => clearTimeout(timer)
	}, [input])

	const params: AdminListParams = { search, page, perPage: PER_PAGE }

	return { input, setInput, page, setPage, params }
}
