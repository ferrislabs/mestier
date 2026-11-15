import { act, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { PublicSurveyFeature } from '#/pages/survey-respond/feature/public-survey-feature'
import { resumeKey } from '#/pages/survey-respond/lib/resume-store'
import {
	photoGraph,
	question,
	respondentSurvey,
} from '#/pages/survey-respond/test/respond-fixtures'
import {
	data,
	hangUntilAborted,
	installServer,
	type RecordedCall,
	refusal,
	renderWithQuery,
} from '#/pages/survey-respond/test/respond-server'

const heicTo = vi.hoisted(() =>
	vi.fn(async () => new Blob(['converted'], { type: 'image/jpeg' })),
)

vi.mock('heic-to', () => ({ heicTo }))

const BASE = '/api/v1/public/surveys/lien-photo'
const TOKEN = 'photo-token-123456'

const PHOTO_BLOCK = 'b0000000-0000-4000-8000-0000000000bb'

const PHOTO_BLOCK_GRAPH = {
	start: 'q-photo',
	nodes: [
		question('q-photo', 'PHOTO', {
			label: 'Photographiez votre ouvrage',
			required: true,
			max_photos: 2,
			block: PHOTO_BLOCK,
		}),
		question('q-caption', 'TEXT', {
			label: 'Légende',
			required: true,
			block: PHOTO_BLOCK,
		}),
		{ type: 'END' as const, id: 'end', message: null },
	],
	edges: [
		{ from: 'q-photo', to: 'q-caption', branch: 'NEXT' as const },
		{ from: 'q-caption', to: 'end', branch: 'NEXT' as const },
	],
}

function routes(
	upload: (
		call: RecordedCall,
	) => ReturnType<typeof data> | Promise<ReturnType<typeof data>>,
	graph: ReturnType<typeof photoGraph> = photoGraph(2),
) {
	return {
		[`GET ${BASE}`]: () => data(respondentSurvey(graph)),
		[`POST ${BASE}/responses`]: () =>
			data(
				{ id: 'r1', answers: {}, status: 'IN_PROGRESS', resume_token: TOKEN },
				201,
			),
		[`POST ${BASE}/responses/${TOKEN}/photos`]: upload,
		[`PUT ${BASE}/responses/${TOKEN}`]: () =>
			data({ id: 'r1', answers: {}, status: 'IN_PROGRESS' }),
		[`POST ${BASE}/responses/${TOKEN}/complete`]: () =>
			data({ id: 'r1', answers: {}, status: 'COMPLETED' }),
	}
}

function jpeg(name: string) {
	return new File(['jpeg-bytes'], name, { type: 'image/jpeg' })
}

const created: string[] = []
const revoked: string[] = []

beforeEach(() => {
	window.localStorage.clear()
	heicTo.mockClear()
	created.length = 0
	revoked.length = 0
	URL.createObjectURL = vi.fn(() => {
		const url = `blob:preview-${created.length + 1}`
		created.push(url)
		return url
	})
	URL.revokeObjectURL = vi.fn((url: string) => {
		revoked.push(url)
	})
})

afterEach(() => {
	vi.useRealTimers()
	vi.unstubAllGlobals()
})

async function picker() {
	await screen.findByRole('heading', { name: 'Photographiez votre ouvrage' })
	return screen.getByLabelText('Choisir des photos')
}

describe('photo answers', () => {
	it('converts a HEIC photo in the browser, then uploads the raw bytes', async () => {
		const { calls } = installServer(routes(() => data({ key: 'k-heic' }, 201)))
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		await userEvent.upload(
			await picker(),
			new File(['heic-bytes'], 'IMG_0001.HEIC', { type: 'image/heic' }),
		)
		await screen.findByText('Envoyée')
		expect(heicTo).toHaveBeenCalledTimes(1)
		const upload = calls.find((call) => call.url.endsWith('/photos'))
		expect(upload?.body).toBeInstanceOf(Blob)
		expect((upload?.body as Blob).type).toBe('image/jpeg')
		expect(upload?.headers.get('Content-Type')).toBe('image/jpeg')
		expect(upload?.headers.has('Authorization')).toBe(false)
		expect(
			screen.getByRole('img', { name: 'Aperçu de IMG_0001.HEIC' }),
		).toBeTruthy()
	})

	it('refuses a HEIC file the browser cannot convert', async () => {
		heicTo.mockRejectedValueOnce(new Error('unsupported'))
		const { calls } = installServer(routes(() => data({ key: 'k' }, 201)))
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		await userEvent.upload(
			await picker(),
			new File(['heic'], 'IMG_0002.heic', { type: 'image/heic' }),
		)
		expect(await screen.findByText(/n'a pas pu être converti/)).toBeTruthy()
		expect(calls.some((call) => call.url.endsWith('/photos'))).toBe(false)
	})

	it('keeps at most max_photos and drops a removed photo from the saved answer, in order', async () => {
		let next = 0
		const { calls } = installServer(
			routes(() => {
				next += 1
				return data({ key: `k${next}` }, 201)
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		await userEvent.upload(await picker(), [
			jpeg('a.jpg'),
			jpeg('b.jpg'),
			jpeg('c.jpg'),
		])
		expect(
			await screen.findByText('Vous pouvez joindre au plus 2 photos.'),
		).toBeTruthy()
		const list = screen.getByRole('list', { name: 'Photos jointes' })
		await waitFor(() =>
			expect(within(list).getAllByText('Envoyée')).toHaveLength(2),
		)
		expect(calls.filter((call) => call.url.endsWith('/photos'))).toHaveLength(2)
		expect(screen.queryByLabelText('Choisir des photos')).toBeNull()

		await userEvent.click(screen.getByRole('button', { name: 'Retirer a.jpg' }))
		expect(revoked).toEqual(['blob:preview-1'])
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		await screen.findByText(/Merci pour vos réponses/)
		const save = calls.find((call) => call.method === 'PUT')
		expect(save?.body).toEqual({ answers: { 'q-photo': { Photos: ['k2'] } } })
	})

	it('explains an upload refusal per file and the per-response cap', async () => {
		let attempt = 0
		installServer(
			routes(() => {
				attempt += 1
				return attempt === 1
					? refusal(413)
					: refusal(409, 'response_photo_cap_reached')
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		await userEvent.upload(await picker(), [jpeg('big.jpg'), jpeg('more.jpg')])
		expect(await screen.findByText('Cette photo est trop lourde.')).toBeTruthy()
		expect(
			await screen.findByText(
				'Vous avez atteint la limite de 20 photos envoyées pour cette réponse.',
			),
		).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(screen.getByText('Ajoutez au moins une photo.')).toBeTruthy()
	})

	it('times out a stalled upload into an error the respondent can remove', async () => {
		vi.useFakeTimers({ shouldAdvanceTime: true })
		const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime })
		installServer(routes(hangUntilAborted))
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		await user.upload(await picker(), jpeg('lent.jpg'))
		expect(await screen.findByText('Envoi…')).toBeTruthy()
		await user.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(
			screen.getByText('Attendez la fin de l’envoi des photos.'),
		).toBeTruthy()
		await act(async () => {
			vi.advanceTimersByTime(60_000)
		})
		expect(
			await screen.findByText(
				'Connexion perdue : la photo n’a pas été envoyée.',
			),
		).toBeTruthy()
		await user.click(screen.getByRole('button', { name: 'Retirer lent.jpg' }))
		expect(screen.queryByText('lent.jpg')).toBeNull()
	})

	it('aborts the upload when a pending photo is removed', async () => {
		const { calls } = installServer(routes(hangUntilAborted))
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		await userEvent.upload(await picker(), jpeg('lent.jpg'))
		await screen.findByText('Envoi…')
		const upload = calls.find((call) => call.url.endsWith('/photos'))
		expect(upload?.signal?.aborted).toBe(false)
		await userEvent.click(
			screen.getByRole('button', { name: 'Retirer lent.jpg' }),
		)
		expect(upload?.signal?.aborted).toBe(true)
		expect(screen.queryByText('lent.jpg')).toBeNull()
		expect(screen.queryByText(/Connexion perdue/)).toBeNull()
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(screen.getByText('Ajoutez au moins une photo.')).toBeTruthy()
	})

	it('starts no response for a file that is not an image', async () => {
		const { calls } = installServer(routes(() => data({ key: 'k' }, 201)))
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		const input = await picker()
		expect(input.getAttribute('aria-describedby')).toContain(
			'answer-q-photo-limit',
		)
		await userEvent.upload(
			input,
			new File(['%PDF'], 'devis.pdf', { type: 'application/pdf' }),
			{ applyAccept: false },
		)
		await waitFor(() =>
			expect(
				within(screen.getByRole('list')).getByText('devis.pdf'),
			).toBeTruthy(),
		)
		expect(calls.map((call) => call.method)).toEqual(['GET'])
	})

	it('labels a photo kept from a resumed response as sent earlier', async () => {
		window.localStorage.setItem(resumeKey('lien-photo'), TOKEN)
		installServer({
			...routes(() => data({ key: 'k' }, 201)),
			[`GET ${BASE}/responses/${TOKEN}`]: () =>
				data({
					id: 'r1',
					answers: { 'q-photo': { Photos: ['k-old'] } },
					status: 'IN_PROGRESS',
				}),
		})
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		expect(await screen.findByText('Photo envoyée précédemment')).toBeTruthy()
	})

	it('uploads the photos of a block member and saves them with the other answers of the screen', async () => {
		const { calls } = installServer(
			routes(() => data({ key: 'k-block' }, 201), PHOTO_BLOCK_GRAPH),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-photo" />)
		await userEvent.click(
			await screen.findByRole('button', { name: 'Terminer' }),
		)
		expect(screen.getByText('Ajoutez au moins une photo.')).toBeTruthy()
		expect(screen.getByText('Cette question est obligatoire.')).toBeTruthy()
		await userEvent.upload(await picker(), jpeg('mur.jpg'))
		await screen.findByText('Envoyée')
		await userEvent.type(
			screen.getByRole('textbox', { name: 'Légende' }),
			'Mur porteur',
		)
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		await waitFor(() =>
			expect(calls.at(-1)?.url).toBe(`${BASE}/responses/${TOKEN}/complete`),
		)
		expect(calls.filter((call) => call.method === 'PUT').at(-1)?.body).toEqual({
			answers: {
				'q-photo': { Photos: ['k-block'] },
				'q-caption': { Text: 'Mur porteur' },
			},
		})
	})
})
