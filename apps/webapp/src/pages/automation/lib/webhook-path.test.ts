import { describe, expect, it } from 'vitest'
import {
	duplicatePathIssue,
	normalizeWebhookPath,
	WEBHOOK_PREFIX,
	webhookPathIssue,
	webhookUrl,
} from '#/pages/automation/lib/webhook-path'

describe('normalizeWebhookPath', () => {
	it.each([
		['Paiement Reçu', 'paiement-recu'],
		['  stripe/Paiement-Recu  ', 'stripe/paiement-recu'],
		['/stripe//paiement/', 'stripe/paiement'],
		['webhooks/paiement-recu', 'paiement-recu'],
		['/Webhooks/Paiement', 'paiement'],
		['webhooks', ''],
		['webhooks-externes', 'webhooks-externes'],
		['a/webhooks/b', 'a/webhooks/b'],
		['Été à Noël', 'ete-a-noel'],
		['', ''],
		['   ', ''],
	])('turns « %s » into « %s »', (input, expected) => {
		expect(normalizeWebhookPath(input)).toBe(expected)
	})

	it('is idempotent', () => {
		const once = normalizeWebhookPath('/Webhooks/Mon Chemin//Ici/')
		expect(normalizeWebhookPath(once)).toBe(once)
	})
})

describe('webhookUrl', () => {
	it('puts the imposed prefix between the origin and the path', () => {
		expect(WEBHOOK_PREFIX).toBe('webhooks')
		expect(webhookUrl('https://mestier.test', 'stripe/paiement-recu')).toBe(
			'https://mestier.test/webhooks/stripe/paiement-recu',
		)
	})
})

describe('webhookPathIssue', () => {
	it('asks for a path when it is empty', () => {
		expect(webhookPathIssue('')).toBe(
			'Indiquez la fin de l’URL après /webhooks/.',
		)
	})

	it.each([
		'Paiement',
		'paiement recu',
		'a//b',
		'/a',
		'a/',
		'a/../b',
		'é',
		'a.b',
	])('refuses « %s » for its characters', (path) => {
		expect(webhookPathIssue(path)).toBe(
			'Lettres minuscules, chiffres, « - », « _ » et « / » uniquement.',
		)
	})

	it.each([
		'paiement-recu',
		'a',
		'a/b/c',
		'factures_2026/payees',
		'0',
		'webhooks-x',
		'x/webhooks/y',
	])('accepts « %s »', (path) => {
		expect(webhookPathIssue(path)).toBeNull()
	})

	it('refuses a path that keeps the imposed prefix, which the server would refuse', () => {
		expect(webhookPathIssue('webhooks/x')).toBe(
			'Le préfixe /webhooks/ est déjà ajouté : retirez-le.',
		)
		expect(webhookPathIssue('webhooks')).toBe(
			'Le préfixe /webhooks/ est déjà ajouté : retirez-le.',
		)
	})

	it('refuses a path longer than 200 characters', () => {
		expect(webhookPathIssue('a'.repeat(200))).toBeNull()
		expect(webhookPathIssue('a'.repeat(201))).toBe(
			'L’URL ne peut pas dépasser 200 caractères.',
		)
	})
})

describe('duplicatePathIssue', () => {
	const holders = [
		{ id: 't1', label: 'Webhook', path: 'paiement-recu' },
		{ id: 't2', label: 'Paiements', path: 'paiement-recu' },
		{ id: 't3', label: 'Autre', path: 'autre' },
	]

	it('names the other node that holds the same path', () => {
		expect(duplicatePathIssue('t1', 'paiement-recu', holders)).toBe(
			'Cette URL est déjà utilisée par « Paiements » dans ce workflow.',
		)
	})

	it('says nothing when the node is alone on its path', () => {
		expect(duplicatePathIssue('t3', 'autre', holders)).toBeNull()
		expect(duplicatePathIssue('t3', 'neuf', holders)).toBeNull()
	})
})
