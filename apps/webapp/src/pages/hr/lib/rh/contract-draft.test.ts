import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	contractDraftOf,
	emptyContractDraft,
	finalizeContractDraft,
	groupContractErrors,
	isContractError,
	isFixedTerm,
} from '#/pages/hr/lib/rh/contract-draft'
import { weekly } from '#/pages/hr/lib/rh/contract-schedule'
import {
	suggestNextContract,
	validateContracts,
} from '#/pages/hr/lib/rh/contracts'
import {
	contractErrors,
	defaultContractSchedule,
	defaultSchedule,
} from '#/pages/hr/lib/rh/members'
import { withLunch } from '#/pages/hr/lib/rh/schedule-draft'

const state = INITIAL_RH_STATE

function memberOf(id: string) {
	const found = state.members.find((member) => member.id === id)
	if (!found) throw new Error(id)
	return found
}

test('isFixedTerm identifie les contrats à durée déterminée', () => {
	assert.equal(isFixedTerm('cdd'), true)
	assert.equal(isFixedTerm('apprentissage'), true)
	assert.equal(isFixedTerm('interim'), true)
	assert.equal(isFixedTerm('cdi'), false)
	assert.equal(isFixedTerm('gerant'), false)
})

test('le brouillon vide est un CDI commençant aujourd’hui avec la semaine par défaut', () => {
	const draft = emptyContractDraft('2026-11-04')
	assert.equal(draft.type, 'cdi')
	assert.equal(draft.startDate, '2026-11-04')
	assert.equal(draft.endDate, null)
	assert.equal(draft.trialEndDate, null)
	assert.deepEqual(draft.schedule, defaultContractSchedule())
	assert.deepEqual(validateContracts([draft]), [])
})

test('contractDraftOf retire l’identifiant et garde les champs', () => {
	const hugo = memberOf('hugo')
	const draft = contractDraftOf(hugo.contracts[0])
	assert.equal('id' in draft, false)
	assert.equal(draft.type, 'cdd')
	assert.equal(draft.endDate, '2026-12-18')
})

test('finalizeContractDraft remplace les dates vides par null', () => {
	const draft = {
		...emptyContractDraft('2026-11-04'),
		type: 'cdd' as const,
		endDate: '',
		trialEndDate: '',
	}
	const result = finalizeContractDraft(draft)
	assert.equal(result.endDate, null)
	assert.equal(result.trialEndDate, null)
	assert.equal(result.startDate, '2026-11-04')
})

test('finalizeContractDraft garde la date de fin d’un CDI', () => {
	const draft = {
		...emptyContractDraft('2026-11-04'),
		type: 'cdi' as const,
		endDate: '2027-01-31',
	}
	assert.equal(finalizeContractDraft(draft).endDate, '2027-01-31')
})

test('groupContractErrors associe chaque message à son champ', () => {
	const draft = {
		...emptyContractDraft('2026-11-04'),
		type: 'cdd' as const,
	}
	const grouped = groupContractErrors(validateContracts([draft]))
	assert.equal(
		grouped.byField.endDate,
		'Ce type de contrat demande une date de fin.',
	)
	assert.equal(grouped.overlap, null)

	const missing = groupContractErrors(
		validateContracts([{ ...draft, startDate: '' }]),
	)
	assert.equal(
		missing.byField.startDate,
		'La date de début du contrat est obligatoire.',
	)

	const inverted = groupContractErrors(
		validateContracts([
			{
				...draft,
				endDate: '2026-10-01',
				trialEndDate: '2026-12-01',
			},
		]),
	)
	assert.equal(
		inverted.byField.endDate,
		'La date de fin précède la date de début.',
	)
	assert.equal(
		inverted.byField.trialEndDate,
		'La fin de la période d’essai doit se situer pendant le contrat.',
	)
})

test('groupContractErrors isole le message de chevauchement', () => {
	const hugo = memberOf('hugo')
	const overlapping = {
		...suggestNextContract(hugo, '2026-11-04'),
		startDate: '2026-12-01',
	}
	const errors = contractErrors(hugo, overlapping)
	const grouped = groupContractErrors(errors)
	assert.ok(
		grouped.overlap?.startsWith('Ce contrat chevauche un autre contrat'),
	)
	assert.deepEqual(grouped.byField, {})
})

test('la suggestion qui suit un CDD de Hugo est valide', () => {
	const hugo = memberOf('hugo')
	const draft = suggestNextContract(hugo, '2026-11-04')
	assert.equal(draft.startDate, '2026-12-19')
	assert.deepEqual(contractErrors(hugo, draft), [])
})

test('contractErrors ignore le contrat en cours d’édition', () => {
	const hugo = memberOf('hugo')
	const contract = hugo.contracts[0]
	assert.deepEqual(
		contractErrors(hugo, contractDraftOf(contract), contract.id),
		[],
	)
	assert.ok(contractErrors(hugo, contractDraftOf(contract)).length > 0)
})

test('isContractError reconnaît les messages du socle', () => {
	assert.equal(
		isContractError('Ce type de contrat demande une date de fin.'),
		true,
	)
	assert.equal(
		isContractError(
			'Ce contrat chevauche un autre contrat (1 janv. au 2 févr.).',
		),
		true,
	)
	assert.equal(isContractError('Le prénom est obligatoire.'), false)
})

test('une pause invalide relevée par validateContracts reste un message libre', () => {
	const draft = {
		...emptyContractDraft('2026-11-04'),
		schedule: weekly(
			withLunch(defaultSchedule(), 0, { start: '13:00', end: '12:00' }),
		),
	}
	const errors = validateContracts([draft])
	assert.deepEqual(errors, ['La fin de la pause doit suivre son début.'])
	assert.equal(isContractError(errors[0]), false)
})
