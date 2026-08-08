import { describe, expect, it } from 'vitest'
import {
	canLoadMoreComments,
	canLoadOlderComments,
	isOwnComment,
	nextPageAfterCreate,
} from '#/pages/planning/lib/comments'

describe('isOwnComment', () => {
	it('is true when the comment author matches the current display name', () => {
		const comment = { author: { id: 'u1', display_name: 'Alice Dupont' } }
		expect(isOwnComment(comment, 'Alice Dupont')).toBe(true)
	})

	it('is false when the comment author is someone else', () => {
		const comment = { author: { id: 'u1', display_name: 'Alice Dupont' } }
		expect(isOwnComment(comment, 'Bob Martin')).toBe(false)
	})

	it('is false when there is no known current user', () => {
		const comment = { author: { id: 'u1', display_name: 'Alice Dupont' } }
		expect(isOwnComment(comment, null)).toBe(false)
	})

	it('is case-sensitive and does not trim — an exact display-name match only', () => {
		const comment = { author: { id: 'u1', display_name: 'Alice Dupont' } }
		expect(isOwnComment(comment, 'alice dupont')).toBe(false)
	})
})

describe('canLoadMoreComments', () => {
	it('is true when the pagination metadata reports a next page', () => {
		expect(
			canLoadMoreComments({
				current_page: 1,
				first_page: 1,
				is_empty: false,
				per_page: 20,
				next_page: 2,
			}),
		).toBe(true)
	})

	it('is false when there is no next page', () => {
		expect(
			canLoadMoreComments({
				current_page: 2,
				first_page: 1,
				is_empty: false,
				per_page: 20,
				next_page: null,
			}),
		).toBe(false)
	})

	it('is false when pagination metadata is absent', () => {
		expect(canLoadMoreComments(null)).toBe(false)
		expect(canLoadMoreComments(undefined)).toBe(false)
	})
})

describe('canLoadOlderComments', () => {
	it('is true while a previous page exists — the thread never loads its whole history up front', () => {
		expect(
			canLoadOlderComments({
				current_page: 2,
				first_page: 1,
				is_empty: false,
				per_page: 20,
				prev_page: 1,
			}),
		).toBe(true)
	})

	it('is false on the first page', () => {
		expect(
			canLoadOlderComments({
				current_page: 1,
				first_page: 1,
				is_empty: false,
				per_page: 20,
				prev_page: null,
			}),
		).toBe(false)
	})
})

describe('nextPageAfterCreate', () => {
	it('stays on page 1 for the very first comment', () => {
		expect(nextPageAfterCreate(null, 20)).toBe(1)
	})

	it('stays on the current last page when it has room for one more', () => {
		expect(
			nextPageAfterCreate(
				{
					current_page: 1,
					first_page: 1,
					is_empty: false,
					per_page: 20,
					last_page: 1,
					total: 5,
				},
				20,
			),
		).toBe(1)
	})

	it('advances to a new page once the last page was exactly full', () => {
		expect(
			nextPageAfterCreate(
				{
					current_page: 1,
					first_page: 1,
					is_empty: false,
					per_page: 20,
					last_page: 1,
					total: 20,
				},
				20,
			),
		).toBe(2)
	})
})
