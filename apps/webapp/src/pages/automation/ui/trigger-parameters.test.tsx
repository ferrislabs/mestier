import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
import {
	type TriggerFieldErrors,
	triggerFieldErrors,
} from '#/pages/automation/lib/validation'
import { credential as fixture } from '#/pages/automation/test/credential-fixtures'
import { TriggerParameters } from '#/pages/automation/ui/trigger-parameters'

for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
	'scrollIntoView',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

const EVENTS: Schemas.EventDescriptorResponse[] = [
	{
		label: 'Devis envoyé',
		name: 'quote.sent',
		payload_example: {},
		subject_kind: 'quote',
		version: 1,
	},
	{
		label: 'Facture payée',
		name: 'invoice.paid',
		payload_example: {},
		subject_kind: 'invoice',
		version: 1,
	},
]

const NO_ERRORS = triggerFieldErrors([])

function trigger(kind: Schemas.TriggerKindDto): Schemas.PlacedTriggerDto {
	return { id: 't1', kind }
}

function renderParameters(
	kind: Schemas.TriggerKindDto,
	handlers: {
		onChooseEvent?: (replacing: string | null) => void
		onRemoveEvent?: (name: string) => void
		onChange?: (kind: Schemas.TriggerKindDto) => void
		onCreateCredential?: () => void
		credentials?: Schemas.CredentialResponse[]
		issues?: NodeIssue[]
		errors?: TriggerFieldErrors
	} = {},
) {
	return render(
		<TriggerParameters
			trigger={trigger(kind)}
			events={EVENTS}
			origin="https://mestier.test"
			credentials={handlers.credentials ?? []}
			credentialHint="Types acceptés : Basic Auth, Bearer, Clé d’API dans un en-tête."
			issues={handlers.issues ?? []}
			errors={handlers.errors ?? NO_ERRORS}
			onChooseEvent={handlers.onChooseEvent ?? vi.fn()}
			onRemoveEvent={handlers.onRemoveEvent ?? vi.fn()}
			onChange={handlers.onChange ?? vi.fn()}
			onCreateCredential={handlers.onCreateCredential ?? vi.fn()}
		/>,
	)
}

describe('TriggerParameters — manual', () => {
	it('renders nothing, as the empty parameters of the prototype', () => {
		const { container } = renderParameters('Manual')

		expect(container.firstChild).toBeNull()
	})
})

describe('TriggerParameters — an event', () => {
	it('shows the event, its name and its version, with the hint of the field', () => {
		renderParameters({ Events: ['quote.sent'] })

		expect(screen.getByText('Événement')).toBeDefined()
		expect(screen.getByText('Devis envoyé')).toBeDefined()
		expect(screen.getByText('quote.sent · version 1')).toBeDefined()
		expect(
			screen.getByText(
				'Le workflow reçoit le payload contractuel de cet événement.',
			),
		).toBeDefined()
	})

	it('changes the only event with the button of the prototype', async () => {
		const user = userEvent.setup()
		const onChooseEvent = vi.fn()
		renderParameters({ Events: ['quote.sent'] }, { onChooseEvent })

		await user.click(
			screen.getByRole('button', { name: 'Changer l’événement' }),
		)

		expect(onChooseEvent).toHaveBeenCalledWith('quote.sent')
	})

	it('names the event on each button once there are several', () => {
		renderParameters({ Events: ['quote.sent', 'invoice.paid'] })

		expect(
			screen.getByRole('button', { name: 'Changer l’événement quote.sent' }),
		).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Retirer l’événement invoice.paid' }),
		).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Ajouter un événement' }),
		).toBeDefined()
	})

	it('offers no removal while a single event remains', () => {
		renderParameters({ Events: ['quote.sent'] })

		expect(screen.queryByRole('button', { name: /Retirer/ })).toBeNull()
	})

	it('invites to choose when no event is set', async () => {
		const user = userEvent.setup()
		const onChooseEvent = vi.fn()
		renderParameters({ Events: [] }, { onChooseEvent })

		expect(screen.getByText('Aucun événement choisi')).toBeDefined()
		await user.click(
			screen.getByRole('button', { name: 'Choisir l’événement' }),
		)

		expect(onChooseEvent).toHaveBeenCalledWith(null)
	})
})

const SCHEDULE: Schemas.TriggerKindDto = {
	Schedule: { frequency: 'Daily', time: '09:00', timezone: 'Europe/Paris' },
}

describe('TriggerParameters — a scheduler', () => {
	it('shows the frequency, the time and the zone with the labels of the prototype', () => {
		renderParameters(SCHEDULE)

		expect(
			screen.getByRole('combobox', { name: 'Fréquence' }).textContent,
		).toBe('Tous les jours')
		expect(
			(screen.getByLabelText('Heure de déclenchement') as HTMLInputElement)
				.value,
		).toBe('09:00')
		expect(
			screen.getByRole('combobox', { name: 'Fuseau horaire' }).textContent,
		).toBe('Europe/Paris')
	})

	it('offers the three frequencies of the prototype', async () => {
		const user = userEvent.setup()
		renderParameters(SCHEDULE)

		await user.click(screen.getByRole('combobox', { name: 'Fréquence' }))

		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['Toutes les heures', 'Tous les jours', 'Toutes les semaines'])
	})

	it('has no weekday unless the schedule is weekly', () => {
		renderParameters(SCHEDULE)

		expect(screen.queryByRole('combobox', { name: 'Jour' })).toBeNull()
	})

	it('asks for the weekday of a weekly schedule, Monday when it has none', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(
			{ Schedule: { frequency: 'Weekly', time: '09:00' } },
			{ onChange },
		)

		expect(screen.getByRole('combobox', { name: 'Jour' }).textContent).toBe(
			'Lundi',
		)
		await user.click(screen.getByRole('combobox', { name: 'Jour' }))
		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual([
			'Lundi',
			'Mardi',
			'Mercredi',
			'Jeudi',
			'Vendredi',
			'Samedi',
			'Dimanche',
		])
		await user.click(screen.getByRole('option', { name: 'Vendredi' }))

		expect(onChange).toHaveBeenLastCalledWith({
			Schedule: { frequency: 'Weekly', time: '09:00', weekday: 'Friday' },
		})
	})

	it('turns to a weekly schedule on Monday and back to a daily one without weekday', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(SCHEDULE, { onChange })

		await user.click(screen.getByRole('combobox', { name: 'Fréquence' }))
		await user.click(
			await screen.findByRole('option', { name: 'Toutes les semaines' }),
		)

		expect(onChange).toHaveBeenLastCalledWith({
			Schedule: {
				frequency: 'Weekly',
				time: '09:00',
				timezone: 'Europe/Paris',
				weekday: 'Monday',
			},
		})
	})

	it('reports the time as it is typed', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(SCHEDULE, { onChange })

		await user.type(screen.getByLabelText('Heure de déclenchement'), '5')

		expect(onChange).toHaveBeenLastCalledWith({
			Schedule: {
				...(SCHEDULE as { Schedule: object }).Schedule,
				time: '09:005',
			},
		})
	})

	it('explains that only the minute counts for an hourly schedule', () => {
		renderParameters({ Schedule: { frequency: 'Hourly', time: '09:15' } })

		expect(screen.getByText(/Seule la minute compte/)).toBeDefined()
	})

	it('picks a zone from the list, or the zone of the organization', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(SCHEDULE, { onChange })

		await user.click(screen.getByRole('combobox', { name: 'Fuseau horaire' }))
		await user.click(await screen.findByText('America/New_York'))
		expect(onChange).toHaveBeenLastCalledWith({
			Schedule: {
				frequency: 'Daily',
				time: '09:00',
				timezone: 'America/New_York',
			},
		})
	})

	it('shows the zone of the organization when the schedule names none, and lets the user go back to it', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(
			{ Schedule: { frequency: 'Daily', time: '09:00', timezone: 'UTC' } },
			{ onChange },
		)

		await user.click(screen.getByRole('combobox', { name: 'Fuseau horaire' }))
		await user.click(await screen.findByText('Fuseau de l’organisation'))

		expect(onChange).toHaveBeenLastCalledWith({
			Schedule: { frequency: 'Daily', time: '09:00', timezone: null },
		})
	})

	it('names the zone of the organization for a schedule that has no zone', () => {
		renderParameters({ Schedule: { frequency: 'Daily', time: '09:00' } })

		expect(
			screen.getByRole('combobox', { name: 'Fuseau horaire' }).textContent,
		).toBe('Fuseau de l’organisation')
	})

	it('shows the issue of the time and the error of the server under the field', () => {
		renderParameters(
			{
				Schedule: { frequency: 'Daily', time: '9h', timezone: 'Mars/Olympus' },
			},
			{
				issues: [
					{
						parameter: 'time',
						message: 'Indiquez une heure au format HH:MM, par exemple 09:00.',
					},
				],
				errors: triggerFieldErrors([
					{
						field: 'timezone',
						message: 'trigger `t1` has the unknown timezone `Mars/Olympus`',
					},
				]),
			},
		)

		expect(
			screen.getAllByRole('alert').map((alert) => alert.textContent),
		).toEqual([
			'Indiquez une heure au format HH:MM, par exemple 09:00.',
			'Ce fuseau horaire n’existe pas.',
		])
		expect(
			screen
				.getByLabelText('Heure de déclenchement')
				.getAttribute('aria-invalid'),
		).toBe('true')
	})

	it('shows an issue once when the server says the same thing', () => {
		const message = 'Indiquez une heure au format HH:MM, par exemple 09:00.'
		renderParameters(
			{ Schedule: { frequency: 'Daily', time: '9h' } },
			{
				issues: [{ parameter: 'time', message }],
				errors: triggerFieldErrors([
					{
						field: 'time',
						message: 'trigger `t1` has the time `9h`, which is not HH:MM',
					},
				]),
			},
		)

		expect(screen.getAllByText(message)).toHaveLength(1)
	})
})

const HOOK: Schemas.TriggerKindDto = {
	Webhook: {
		method: 'POST',
		path: 'paiement-recu',
		credential_id: null,
		contract: [],
		on_invalid: 'Reject',
	},
}

function credential(
	id: string,
	name: string,
	kind = 'http_header',
): Schemas.CredentialResponse {
	return fixture({ id, name, kind })
}

describe('TriggerParameters — a webhook', () => {
	it('shows its fields in the order of the prototype', () => {
		renderParameters(HOOK)

		const labels = [
			'Méthode',
			'URL',
			'Identification',
			'Contrat du corps reçu',
			'Requête non conforme au contrat',
		].map((label) => screen.getByText(label))
		for (let index = 1; index < labels.length; index += 1) {
			expect(
				(labels[index - 1] as HTMLElement).compareDocumentPosition(
					labels[index] as HTMLElement,
				) & Node.DOCUMENT_POSITION_FOLLOWING,
			).toBeTruthy()
		}
	})

	it('offers the five methods and reports the chosen one', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(HOOK, { onChange })

		expect(screen.getByRole('combobox', { name: 'Méthode' }).textContent).toBe(
			'POST',
		)
		await user.click(screen.getByRole('combobox', { name: 'Méthode' }))
		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['GET', 'POST', 'PUT', 'PATCH', 'DELETE'])
		await user.click(screen.getByRole('option', { name: 'PUT' }))

		expect(onChange).toHaveBeenLastCalledWith({
			Webhook: { ...(HOOK as { Webhook: object }).Webhook, method: 'PUT' },
		})
	})

	it('shows the URL under its prefix with the hint of the prototype', () => {
		renderParameters(HOOK)

		expect(screen.getByTitle('Préfixe imposé').textContent).toBe('/webhooks/')
		expect(
			screen.getByText('https://mestier.test/webhooks/paiement-recu'),
		).toBeDefined()
		expect(
			screen.getByText(
				'Le préfixe /webhooks/ est imposé. La suite est libre : lettres minuscules, chiffres, « - », « _ » et « / ».',
			),
		).toBeDefined()
	})

	it('reports the path normalized when the field is left', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(
			{ Webhook: { method: 'POST', path: ' Mon Chemin/ ' } },
			{ onChange },
		)

		await user.click(screen.getByPlaceholderText('stripe/paiement-recu'))
		await user.tab()

		expect(onChange).toHaveBeenLastCalledWith({
			Webhook: { method: 'POST', path: 'mon-chemin' },
		})
	})

	it('shows the issues of the path and the 409 of the server on the field', () => {
		renderParameters(HOOK, {
			issues: [
				{
					parameter: 'path',
					message:
						'Cette URL est déjà utilisée par « Doublon » dans ce workflow.',
				},
			],
			errors: triggerFieldErrors([
				{
					field: 'path',
					message:
						'trigger `t1` uses a webhook path another workflow already holds',
				},
			]),
		})

		expect(
			screen.getAllByRole('alert').map((alert) => alert.textContent),
		).toEqual([
			'Cette URL est déjà utilisée par « Doublon » dans ce workflow.',
			'Ce chemin est déjà utilisé.',
		])
		expect(
			screen
				.getByPlaceholderText('stripe/paiement-recu')
				.getAttribute('aria-invalid'),
		).toBe('true')
	})

	it('warns that anyone with the URL can start the workflow while no identification is chosen', () => {
		renderParameters(HOOK, { credentials: [credential('c1', 'Clé Stripe')] })

		expect(
			screen.getByText(
				'Sans identification, toute personne qui connaît cette URL peut déclencher ce workflow.',
			),
		).toBeDefined()
	})

	it('drops the warning once an identification is chosen', () => {
		renderParameters(
			{ Webhook: { method: 'POST', path: 'x', credential_id: 'c1' } },
			{ credentials: [credential('c1', 'Clé Stripe')] },
		)

		expect(screen.queryByText(/Sans identification/)).toBeNull()
		expect(
			screen.getByRole('combobox', { name: 'Identification' }).textContent,
		).toBe('Clé Stripe')
	})

	it('shows the filter hint and offers to create an identification', async () => {
		const user = userEvent.setup()
		const onCreateCredential = vi.fn()
		renderParameters(HOOK, { onCreateCredential })

		expect(
			screen.getByText(
				'Types acceptés : Basic Auth, Bearer, Clé d’API dans un en-tête.',
			),
		).toBeDefined()
		await user.click(
			screen.getByRole('button', { name: 'Créer une identification' }),
		)

		expect(onCreateCredential).toHaveBeenCalledOnce()
	})

	it('chooses an identification, or none', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(
			{ Webhook: { method: 'POST', path: 'x', credential_id: 'c1' } },
			{
				onChange,
				credentials: [
					credential('c1', 'Clé Stripe'),
					credential('c2', 'Basic'),
				],
			},
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(await screen.findByRole('option', { name: 'Basic' }))
		expect(onChange).toHaveBeenLastCalledWith({
			Webhook: { method: 'POST', path: 'x', credential_id: 'c2' },
		})

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('option', { name: 'Aucune identification' }),
		)
		expect(onChange).toHaveBeenLastCalledWith({
			Webhook: { method: 'POST', path: 'x', credential_id: null },
		})
	})

	it('shows what the server says about the identification', () => {
		renderParameters(HOOK, {
			errors: triggerFieldErrors([
				{
					field: 'credential',
					message: 'trigger `t1` references unknown credential `c9`',
				},
			]),
		})

		expect(
			screen.getByText('Cette identification n’existe plus.'),
		).toBeDefined()
	})

	it('edits the contract, with the hint of the prototype', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(HOOK, { onChange })

		expect(
			screen.getByText('Les nœuds suivants utilisent ces clés typées.'),
		).toBeDefined()
		await user.click(screen.getByRole('button', { name: 'Ajouter un champ' }))

		expect(onChange).toHaveBeenLastCalledWith({
			Webhook: {
				...(HOOK as { Webhook: object }).Webhook,
				contract: [{ name: '', kind: 'Text', required: true, example: '' }],
			},
		})
	})

	it('shows the contract error that has no key', () => {
		renderParameters(HOOK, {
			errors: triggerFieldErrors([
				{
					field: 'contract',
					message: 'trigger `t1` has a contract key with no name',
				},
			]),
		})

		expect(screen.getByText('Donnez un nom à cette clé.')).toBeDefined()
	})

	it('refuses or accepts a body that breaks the contract', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderParameters(HOOK, { onChange })

		expect(
			screen.getByRole('combobox', { name: 'Requête non conforme au contrat' })
				.textContent,
		).toBe('Refuser (réponse 400)')
		await user.click(
			screen.getByRole('combobox', { name: 'Requête non conforme au contrat' }),
		)
		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['Refuser (réponse 400)', 'Accepter et lancer le workflow'])
		await user.click(
			screen.getByRole('option', { name: 'Accepter et lancer le workflow' }),
		)

		expect(onChange).toHaveBeenLastCalledWith({
			Webhook: {
				...(HOOK as { Webhook: object }).Webhook,
				on_invalid: 'Accept',
			},
		})
	})

	it('reads a webhook saved without its optional fields as the API does', () => {
		renderParameters({ Webhook: { method: 'GET', path: 'x' } })

		expect(
			screen.getByRole('combobox', { name: 'Requête non conforme au contrat' })
				.textContent,
		).toBe('Refuser (réponse 400)')
		expect(screen.getByText(/Aucun champ décrit/)).toBeDefined()
	})
})
