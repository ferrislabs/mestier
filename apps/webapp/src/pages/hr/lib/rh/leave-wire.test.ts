import { expect, test } from 'vitest'
import {
	ADJUSTMENT_KIND_FROM_WIRE,
	ADJUSTMENT_KIND_TO_WIRE,
	accountFailureOf,
	accountOf,
	accountRequestOf,
	adjustmentFailureOf,
	adjustmentOf,
	adjustmentRequestOf,
	overlayAccounts,
	overlayAdjustments,
} from '#/pages/hr/lib/rh/leave-wire'
import type { AdjustmentDraft, AdjustmentKind } from '#/pages/hr/lib/rh/types'
import {
	accountRecord,
	adjustmentRecord,
	DEFAULT_ACCOUNT_RECORDS,
} from '#/test/rh-leave-fixtures'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const UNPROCESSABLE = (field: string, message = '') => ({
	status: 422,
	data: { message, details: { field } },
})

test('accountOf reads a monthly and an annual account with their real id', () => {
	const [cp, rtt] = DEFAULT_ACCOUNT_RECORDS
	expect(accountOf(cp ?? accountRecord('x', 'x'))).toEqual({
		id: 'acc-cp',
		name: 'Congés payés',
		acquisition: { kind: 'monthly', perMonth: 2.08 },
		periodStartMonth: 6,
		carryOverMax: 5,
		archived: false,
	})
	expect(accountOf(rtt ?? accountRecord('x', 'x')).acquisition).toEqual({
		kind: 'annual',
		days: 10,
	})
	expect(
		accountOf(accountRecord('a', 'A', { archived_at: '2026-11-01T00:00:00Z' }))
			.archived,
	).toBe(true)
})

test('an account survives the round trip through its request', () => {
	for (const record of DEFAULT_ACCOUNT_RECORDS) {
		const account = accountOf(record)
		const acquisition = account.acquisition
		const request = accountRequestOf(account, {
			kind: acquisition.kind,
			value:
				acquisition.kind === 'monthly'
					? acquisition.perMonth
					: acquisition.days,
			periodStartMonth: account.periodStartMonth,
			carryOverMax: account.carryOverMax,
		})
		expect(request).toEqual({
			name: record.name,
			acquisition: record.acquisition,
			period_start_month: record.period_start_month,
			carry_over_max: record.carry_over_max,
		})
	}
})

test('an edited rule switches the acquisition kind on the wire', () => {
	const account = accountOf(accountRecord('a', 'Ancienneté'))
	expect(
		accountRequestOf(account, {
			kind: 'annual',
			value: 3,
			periodStartMonth: 9,
			carryOverMax: 0,
		}),
	).toEqual({
		name: 'Ancienneté',
		acquisition: { kind: 'ANNUAL', days: 3 },
		period_start_month: 9,
		carry_over_max: 0,
	})
})

test('adjustment kinds convert both ways', () => {
	for (const kind of ['adjustment', 'carry_over'] as AdjustmentKind[])
		expect(ADJUSTMENT_KIND_FROM_WIRE[ADJUSTMENT_KIND_TO_WIRE[kind]]).toBe(kind)
})

test('an adjustment survives the round trip through its request', () => {
	const record = adjustmentRecord('adj-1', 'u-1', 'acc-cp', {
		days: -0.5,
		author_member_id: 'u-2',
	})
	const adjustment = adjustmentOf(record)
	expect(adjustment).toEqual({
		id: 'adj-1',
		memberId: 'u-1',
		accountId: 'acc-cp',
		date: '2026-11-02',
		days: -0.5,
		kind: 'adjustment',
		reason: 'Jour récupéré',
		authorId: 'u-2',
	})
	expect(adjustmentRequestOf(adjustment)).toEqual({
		member_id: 'u-1',
		account_id: 'acc-cp',
		date: '2026-11-02',
		days: -0.5,
		kind: 'ADJUSTMENT',
		reason: 'Jour récupéré',
	})
})

test('a carry-over without a reason is sent with the default one', () => {
	const draft: AdjustmentDraft = {
		memberId: 'u-1',
		accountId: 'acc-cp',
		date: '2026-11-02',
		days: 3,
		kind: 'carry_over',
		reason: '  ',
	}
	expect(adjustmentRequestOf(draft).reason).toBe('Report')
	expect(
		adjustmentRequestOf({ ...draft, reason: ' Report 2025 ' }).reason,
	).toBe('Report 2025')
})

test('the overlay replaces the accounts and the adjustments of the state', () => {
	const withAccounts = overlayAccounts(
		INITIAL_RH_STATE,
		DEFAULT_ACCOUNT_RECORDS,
	)
	expect(withAccounts.accounts.map((account) => account.id)).toEqual([
		'acc-cp',
		'acc-rtt',
	])
	const withAdjustments = overlayAdjustments(withAccounts, [
		adjustmentRecord('adj-1', 'u-1', 'acc-cp'),
	])
	expect(
		withAdjustments.adjustments.map((adjustment) => adjustment.id),
	).toEqual(['adj-1'])
	expect(overlayAdjustments(withAccounts, []).adjustments).toEqual([])
	expect(withAdjustments.members).toBe(INITIAL_RH_STATE.members)
})

test('a refused account names the field', () => {
	expect(accountFailureOf(UNPROCESSABLE('acquisition')).field).toBe('value')
	expect(accountFailureOf(UNPROCESSABLE('period_start_month')).field).toBe(
		'periodStartMonth',
	)
	expect(accountFailureOf(UNPROCESSABLE('carry_over_max')).field).toBe(
		'carryOverMax',
	)
	expect(accountFailureOf({ status: 404 }).field).toBeNull()
	expect(accountFailureOf(new Error('réseau')).message).toBe('réseau')
})

test('a refused adjustment explains the days, the reason and the account', () => {
	expect(adjustmentFailureOf(UNPROCESSABLE('days')).field).toBe('days')
	expect(
		adjustmentFailureOf(
			UNPROCESSABLE(
				'days',
				'days: the carry-over exceeds the maximum of the account',
			),
		).message,
	).toMatch(/dépasse le maximum/)
	expect(
		adjustmentFailureOf(
			UNPROCESSABLE('days', 'days: a carry-over is always positive'),
		).message,
	).toMatch(/toujours positif/)
	expect(adjustmentFailureOf(UNPROCESSABLE('reason')).field).toBe('reason')
	expect(adjustmentFailureOf(UNPROCESSABLE('account_id')).message).toMatch(
		/archivé/,
	)
	expect(adjustmentFailureOf(UNPROCESSABLE('member_id')).field).toBe('member')
	expect(adjustmentFailureOf({ status: 404 }).field).toBeNull()
})
