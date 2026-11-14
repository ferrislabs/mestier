import { useQuery } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { prepareFile } from '#/pages/photo-library/lib/files'
import { fromSchema, identifierQuestion } from '#/pages/survey/lib/survey-graph'
import type { EditorGraph, QuestionNode } from '#/pages/survey/types'
import type { RespondSource } from '#/pages/survey-respond/feature/respond-sources'
import {
	type AnswerMap,
	buildAnswers,
	type Draft,
	draftsFromAnswers,
	isBlank,
	MESSAGES,
	maxPhotos,
	tooManyPhotosMessage,
	validateDraft,
} from '#/pages/survey-respond/lib/answer-draft'
import {
	classifyFailure,
	type Failure,
	IDENTIFIER_TAKEN,
	NETWORK,
	photoErrorMessage,
	RATE_LIMITED,
} from '#/pages/survey-respond/lib/respond-errors'
import {
	knownLength,
	respondPath,
} from '#/pages/survey-respond/lib/respond-path'
import type {
	PhotoItem,
	RespondentView,
	RespondPhase,
	SaveState,
	StepProgress,
} from '#/pages/survey-respond/lib/respond-types'

export const DEFAULT_END_MESSAGE =
	'Merci pour vos réponses ! Elles ont bien été enregistrées.'

type Override = Extract<
	RespondPhase,
	{ name: 'expired' | 'blocked' | 'completed' | 'unavailable' }
>

export interface RespondentOptions {
	queryKey: readonly string[]
	convertHeic: (file: File) => Promise<Blob>
}

function isPending(item: PhotoItem): boolean {
	return item.status === 'converting' || item.status === 'uploading'
}

function photoKeys(items: readonly PhotoItem[] | undefined): string[] {
	return (items ?? []).flatMap((item) =>
		item.status === 'done' && item.key ? [item.key] : [],
	)
}

function retryMessage(failure: Failure): string {
	return failure.kind === 'rate_limited' ? RATE_LIMITED : NETWORK
}

const UPLOAD_TIMEOUT = 60_000

function uploadSignal(controller: AbortController): {
	signal: AbortSignal
	release: () => void
} {
	if (
		typeof AbortSignal.any === 'function' &&
		typeof AbortSignal.timeout === 'function'
	) {
		return {
			signal: AbortSignal.any([
				controller.signal,
				AbortSignal.timeout(UPLOAD_TIMEOUT),
			]),
			release: () => undefined,
		}
	}
	const timer = setTimeout(() => controller.abort(), UPLOAD_TIMEOUT)
	return { signal: controller.signal, release: () => clearTimeout(timer) }
}

let photoSequence = 0

function nextPhotoId(): string {
	photoSequence += 1
	return `photo-${photoSequence}`
}

export function useSurveyRespondent(
	source: RespondSource,
	{ queryKey, convertHeic }: RespondentOptions,
) {
	const surveyQuery = useQuery({
		queryKey: [...queryKey, 'survey'],
		queryFn: source.loadSurvey,
		retry: false,
		refetchOnWindowFocus: false,
		staleTime: Number.POSITIVE_INFINITY,
	})
	const restoreQuery = useQuery({
		queryKey: [...queryKey, 'restore'],
		queryFn: source.restore,
		enabled: surveyQuery.isSuccess,
		retry: false,
		refetchOnWindowFocus: false,
		staleTime: Number.POSITIVE_INFINITY,
		gcTime: 0,
	})

	const survey = surveyQuery.data ?? null
	const graph: EditorGraph | null = useMemo(
		() => (survey ? fromSchema(survey.graph) : null),
		[survey],
	)
	const questions = useMemo(
		() =>
			(graph?.nodes ?? []).filter(
				(node): node is QuestionNode => node.type === 'QUESTION',
			),
		[graph],
	)

	const [override, setOverride] = useState<Override | null>(null)
	const [fresh, setFresh] = useState(false)
	const [initialized, setInitialized] = useState(false)
	const [drafts, setDrafts] = useState<Record<string, Draft>>({})
	const [photos, setPhotoState] = useState<Record<string, PhotoItem[]>>({})
	const [photoNotice, setPhotoNotice] = useState<Record<string, string>>({})
	const [index, setIndex] = useState(0)
	const [stepError, setStepError] = useState<string | null>(null)
	const [actionError, setActionError] = useState<string | null>(null)
	const [busy, setBusy] = useState(false)
	const [saveState, setSaveState] = useState<SaveState>('idle')
	const [saveMessage, setSaveMessage] = useState<string | null>(null)
	const [stepChange, setStepChange] = useState(0)

	const headingRef = useRef<HTMLHeadingElement>(null)
	const handleRef = useRef<string | null>(null)
	const startRef = useRef<Promise<string | null> | null>(null)
	const saveChain = useRef<Promise<boolean>>(Promise.resolve(true))
	const latestAnswers = useRef<AnswerMap | null>(null)
	const objectUrls = useRef(new Set<string>())
	const photosRef = useRef<Record<string, PhotoItem[]>>({})
	const uploads = useRef(new Map<string, AbortController>())

	const updatePhotos = useCallback(
		(
			change: (
				previous: Record<string, PhotoItem[]>,
			) => Record<string, PhotoItem[]>,
		) => {
			photosRef.current = change(photosRef.current)
			setPhotoState(photosRef.current)
		},
		[],
	)

	const effectiveDrafts = useMemo(() => {
		const merged: Record<string, Draft> = { ...drafts }
		for (const node of questions) {
			if (node.kind === 'PHOTO') merged[node.id] = photoKeys(photos[node.id])
		}
		return merged
	}, [drafts, photos, questions])

	const path = useMemo(
		() =>
			graph
				? respondPath(graph, buildAnswers(questions, effectiveDrafts))
				: { questions: [], end: null },
		[graph, questions, effectiveDrafts],
	)

	const restored = fresh ? null : (restoreQuery.data ?? null)

	useEffect(() => {
		if (initialized || !graph || !restoreQuery.isSuccess) return
		setInitialized(true)
		if (!restored) return
		handleRef.current = restored.handle
		const merged = draftsFromAnswers(questions, restored.answers)
		const restoredDrafts: Record<string, Draft> = {}
		const restoredPhotos: Record<string, PhotoItem[]> = {}
		for (const node of questions) {
			const value = merged[node.id]
			if (value === undefined) continue
			if (node.kind !== 'PHOTO') {
				restoredDrafts[node.id] = value
				continue
			}
			restoredPhotos[node.id] = (Array.isArray(value) ? value : []).map(
				(key, position) => ({
					id: `saved-${node.id}-${position}`,
					name: `Photo ${position + 1}`,
					status: 'done',
					previewUrl: null,
					key,
					message: 'Photo envoyée précédemment',
				}),
			)
		}
		setDrafts(restoredDrafts)
		updatePhotos(() => restoredPhotos)
		const resumedPath = respondPath(graph, buildAnswers(questions, merged))
		const firstOpen = resumedPath.questions.findIndex((node) =>
			isBlank(node, merged[node.id]),
		)
		setIndex(
			firstOpen === -1
				? Math.max(resumedPath.questions.length - 1, 0)
				: firstOpen,
		)
	}, [
		initialized,
		graph,
		restoreQuery.isSuccess,
		restored,
		questions,
		updatePhotos,
	])

	useEffect(() => {
		if (!restoreQuery.isError || fresh) return
		const failure = classifyFailure(restoreQuery.error, 'response')
		if (failure.kind === 'expired') source.forget()
	}, [restoreQuery.isError, restoreQuery.error, fresh, source])

	useEffect(() => {
		if (restored?.status === 'COMPLETED') source.forget()
	}, [restored, source])

	useEffect(() => {
		if (!survey) return
		const previous = document.title
		document.title = `${survey.title} · Mestier`
		return () => {
			document.title = previous
		}
	}, [survey])

	useEffect(() => {
		if (stepChange === 0 && override === null) return
		headingRef.current?.focus()
	}, [stepChange, override])

	useEffect(() => {
		const urls = objectUrls.current
		const pending = uploads.current
		return () => {
			for (const controller of pending.values()) controller.abort()
			pending.clear()
			for (const url of urls) URL.revokeObjectURL(url)
			urls.clear()
		}
	}, [])

	const phase: RespondPhase = useMemo(() => {
		if (override) return override
		if (surveyQuery.isPending) return { name: 'loading' }
		if (surveyQuery.isError) {
			const failure = classifyFailure(surveyQuery.error, 'survey')
			if (failure.kind === 'unavailable') return { name: 'unavailable' }
			if (failure.kind === 'blocked') {
				return { name: 'blocked', reason: failure.reason }
			}
			return { name: 'retry', message: retryMessage(failure) }
		}
		if (!fresh) {
			if (restoreQuery.isPending) return { name: 'loading' }
			if (restoreQuery.isError) {
				const failure = classifyFailure(restoreQuery.error, 'response')
				if (failure.kind === 'expired') return { name: 'expired' }
				if (failure.kind === 'blocked') {
					return { name: 'blocked', reason: failure.reason }
				}
				if (failure.kind === 'unavailable') return { name: 'unavailable' }
				return { name: 'retry', message: retryMessage(failure) }
			}
			if (restored?.status === 'COMPLETED') {
				return { name: 'blocked', reason: 'response_already_completed' }
			}
		}
		if (!initialized) return { name: 'loading' }
		return { name: 'answering' }
	}, [
		override,
		surveyQuery.isPending,
		surveyQuery.isError,
		surveyQuery.error,
		fresh,
		restoreQuery.isPending,
		restoreQuery.isError,
		restoreQuery.error,
		restored,
		initialized,
	])

	const position = Math.min(index, Math.max(path.questions.length - 1, 0))
	const current = path.questions[position] ?? null
	const isLast = position === path.questions.length - 1
	const progress: StepProgress | null =
		current && graph
			? {
					position: position + 1,
					total: knownLength(graph, current.id, path.questions.length),
				}
			: null

	const fail = useCallback(
		(failure: Failure): boolean => {
			switch (failure.kind) {
				case 'blocked':
					setOverride({ name: 'blocked', reason: failure.reason })
					return true
				case 'expired':
					source.forget()
					setOverride({ name: 'expired' })
					return true
				case 'unavailable':
					setOverride({ name: 'unavailable' })
					return true
				default:
					return false
			}
		},
		[source],
	)

	const ensureHandle = useCallback(async (): Promise<string | null> => {
		if (handleRef.current) return handleRef.current
		if (startRef.current) return startRef.current
		const starting = source
			.start()
			.then((response) => {
				handleRef.current = response.handle
				return response.handle as string | null
			})
			.catch((error: unknown) => {
				const failure = classifyFailure(error, 'survey')
				if (!fail(failure)) {
					setActionError(
						failure.kind === 'invalid'
							? failure.message
							: retryMessage(failure),
					)
				}
				return null
			})
			.finally(() => {
				startRef.current = null
			})
		startRef.current = starting
		return starting
	}, [source, fail])

	const flush = useCallback((): Promise<boolean> => {
		const run = async (): Promise<boolean> => {
			const target = handleRef.current
			const snapshot = latestAnswers.current
			if (!target || !snapshot) return true
			try {
				await source.save(target, snapshot)
				if (latestAnswers.current === snapshot) {
					setSaveState('saved')
					setSaveMessage(null)
				}
				return true
			} catch (error: unknown) {
				const failure = classifyFailure(error, 'response')
				if (fail(failure)) return false
				setSaveState('error')
				setSaveMessage(
					failure.kind === 'invalid' ? failure.message : retryMessage(failure),
				)
				return false
			}
		}
		saveChain.current = saveChain.current.then(run, run)
		return saveChain.current
	}, [source, fail])

	const persist = useCallback(
		(answers: AnswerMap): Promise<boolean> => {
			latestAnswers.current = answers
			setSaveState('saving')
			setSaveMessage(null)
			return flush()
		},
		[flush],
	)

	const goTo = useCallback((target: number) => {
		setIndex(target)
		setStepError(null)
		setActionError(null)
		setStepChange((count) => count + 1)
	}, [])

	const pendingFor = useCallback(
		(questionId: string) => (photos[questionId] ?? []).filter(isPending).length,
		[photos],
	)

	const pendingAnywhere = useCallback(
		() =>
			path.questions.some((node) => (photos[node.id] ?? []).some(isPending)),
		[photos, path.questions],
	)

	const setDraft = useCallback((questionId: string, draft: Draft) => {
		setDrafts((previous) => ({ ...previous, [questionId]: draft }))
		setStepError(null)
	}, [])

	const next = useCallback(async () => {
		if (!current || busy) return
		const problem = validateDraft(
			current,
			effectiveDrafts[current.id],
			pendingFor(current.id),
		)
		if (problem) {
			setStepError(problem)
			return
		}
		if (isLast && pendingAnywhere()) {
			setActionError(MESSAGES.photoPending)
			return
		}
		setBusy(true)
		setActionError(null)
		try {
			const target = await ensureHandle()
			if (!target) return
			const answers = buildAnswers(path.questions, effectiveDrafts)
			if (!isLast) {
				void persist(answers)
				goTo(position + 1)
				return
			}
			if (!path.end) {
				setActionError(
					'Ce questionnaire est incomplet : il ne peut pas être envoyé.',
				)
				return
			}
			const saved = await persist(answers)
			if (!saved) return
			try {
				await source.complete(target)
				source.forget()
				setOverride({
					name: 'completed',
					message: path.end.message?.trim() || DEFAULT_END_MESSAGE,
				})
			} catch (error: unknown) {
				const failure = classifyFailure(error, 'response')
				if (
					source.mode === 'public' &&
					failure.kind === 'blocked' &&
					failure.reason === 'respondent_already_answered' &&
					graph
				) {
					const identifier = identifierQuestion(graph)
					const found = identifier
						? path.questions.findIndex((node) => node.id === identifier.id)
						: -1
					if (found >= 0) {
						goTo(found)
						setStepError(IDENTIFIER_TAKEN)
						return
					}
				}
				if (fail(failure)) return
				if (failure.kind === 'invalid') {
					const found = failure.questionId
						? path.questions.findIndex((node) => node.id === failure.questionId)
						: -1
					if (found >= 0) {
						goTo(found)
						setStepError(failure.message)
						return
					}
					setActionError(failure.message)
					return
				}
				setActionError(retryMessage(failure))
			}
		} finally {
			setBusy(false)
		}
	}, [
		current,
		busy,
		effectiveDrafts,
		pendingFor,
		pendingAnywhere,
		ensureHandle,
		graph,
		path,
		isLast,
		persist,
		goTo,
		position,
		source,
		fail,
	])

	const back = useCallback(() => {
		if (position === 0 || busy) return
		if (handleRef.current) {
			void persist(buildAnswers(path.questions, effectiveDrafts))
		}
		goTo(position - 1)
	}, [position, busy, persist, path.questions, effectiveDrafts, goTo])

	const retrySave = useCallback(() => {
		setSaveState('saving')
		setSaveMessage(null)
		void flush()
	}, [flush])

	const patchPhoto = useCallback(
		(questionId: string, id: string, patch: Partial<PhotoItem>) => {
			updatePhotos((previous) => ({
				...previous,
				[questionId]: (previous[questionId] ?? []).map((item) =>
					item.id === id ? { ...item, ...patch } : item,
				),
			}))
		},
		[updatePhotos],
	)

	const addPhotos = useCallback(
		async (questionId: string, files: readonly File[]) => {
			const node = questions.find((candidate) => candidate.id === questionId)
			if (!node || files.length === 0) return
			const taken = (photosRef.current[questionId] ?? []).filter(
				(item) => item.status !== 'error',
			).length
			const room = Math.max(maxPhotos(node) - taken, 0)
			const accepted = files.slice(0, room)
			setPhotoNotice((previous) => {
				const { [questionId]: _, ...rest } = previous
				return files.length > room
					? { ...rest, [questionId]: tooManyPhotosMessage(maxPhotos(node)) }
					: rest
			})
			if (accepted.length === 0) return
			setStepError(null)
			const items = accepted.map((file) => {
				const controller = new AbortController()
				const item: PhotoItem = {
					id: nextPhotoId(),
					name: file.name,
					status: 'converting',
					previewUrl: null,
					key: null,
					message: null,
				}
				uploads.current.set(item.id, controller)
				return { file, item, controller }
			})
			updatePhotos((previous) => ({
				...previous,
				[questionId]: [
					...(previous[questionId] ?? []),
					...items.map(({ item }) => item),
				],
			}))
			for (const { file, item, controller } of items) {
				if (!uploads.current.has(item.id)) continue
				const prepared = await prepareFile(file, convertHeic)
				if (!uploads.current.has(item.id)) continue
				if (prepared.kind === 'rejected') {
					uploads.current.delete(item.id)
					patchPhoto(questionId, item.id, {
						status: 'error',
						message: prepared.message,
					})
					continue
				}
				const target = await ensureHandle()
				if (!uploads.current.has(item.id)) continue
				if (!target) {
					uploads.current.delete(item.id)
					patchPhoto(questionId, item.id, {
						status: 'error',
						message: 'La photo n’a pas pu être envoyée.',
					})
					continue
				}
				const previewUrl = URL.createObjectURL(prepared.file)
				objectUrls.current.add(previewUrl)
				patchPhoto(questionId, item.id, { status: 'uploading', previewUrl })
				const { signal, release } = uploadSignal(controller)
				try {
					const key = await source.uploadPhoto(target, prepared.file, signal)
					patchPhoto(questionId, item.id, { status: 'done', key })
				} catch (error: unknown) {
					if (!uploads.current.has(item.id)) continue
					if (fail(classifyFailure(error, 'response'))) return
					patchPhoto(questionId, item.id, {
						status: 'error',
						message: photoErrorMessage(error),
					})
				} finally {
					release()
					uploads.current.delete(item.id)
				}
			}
		},
		[
			questions,
			updatePhotos,
			ensureHandle,
			patchPhoto,
			convertHeic,
			source,
			fail,
		],
	)

	const removePhoto = useCallback(
		(questionId: string, id: string) => {
			uploads.current.get(id)?.abort()
			uploads.current.delete(id)
			const removed = (photosRef.current[questionId] ?? []).find(
				(item) => item.id === id,
			)
			if (removed?.previewUrl) {
				URL.revokeObjectURL(removed.previewUrl)
				objectUrls.current.delete(removed.previewUrl)
			}
			updatePhotos((previous) => ({
				...previous,
				[questionId]: (previous[questionId] ?? []).filter(
					(item) => item.id !== id,
				),
			}))
			setPhotoNotice((previous) => {
				const { [questionId]: _, ...rest } = previous
				return rest
			})
		},
		[updatePhotos],
	)

	const restart = useCallback(() => {
		source.forget()
		handleRef.current = null
		latestAnswers.current = null
		for (const controller of uploads.current.values()) controller.abort()
		uploads.current.clear()
		setDrafts({})
		updatePhotos(() => ({}))
		setPhotoNotice({})
		setIndex(0)
		setStepError(null)
		setActionError(null)
		setSaveState('idle')
		setFresh(true)
		setInitialized(true)
		setOverride(null)
		setStepChange((count) => count + 1)
	}, [source, updatePhotos])

	const reload = useCallback(() => {
		if (surveyQuery.isError) void surveyQuery.refetch()
		else void restoreQuery.refetch()
	}, [surveyQuery, restoreQuery])

	const view: RespondentView = {
		phase,
		survey,
		current,
		draft: current ? effectiveDrafts[current.id] : undefined,
		photos: current ? (photos[current.id] ?? []) : [],
		photoNotice: current ? (photoNotice[current.id] ?? null) : null,
		progress,
		isFirst: position === 0,
		isLast,
		busy,
		stepError,
		actionError,
		saveState,
		saveMessage,
		headingRef,
		setDraft,
		next,
		back,
		retrySave,
		addPhotos,
		removePhoto,
		restart,
		reload,
	}
	return view
}
